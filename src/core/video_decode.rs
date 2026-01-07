use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use ffmpeg_next as ffmpeg;
use image::RgbaImage;

const AV_TIME_BASE: i64 = 1_000_000;
const MAX_DECODERS: usize = 8;
const MAX_SEQUENTIAL_JUMP_SECONDS: f64 = 2.0;

struct DecodeRequest {
    path: PathBuf,
    time_seconds: f64,
    mode: DecodeMode,
    respond_to: mpsc::Sender<Option<RgbaImage>>,
}

#[derive(Clone, Copy, Debug)]
pub enum DecodeMode {
    Seek,
    Sequential,
}

/// A dedicated worker for in-process video decoding with FFmpeg.
#[derive(Clone)]
pub struct VideoDecodeWorker {
    sender: mpsc::Sender<DecodeRequest>,
}

impl VideoDecodeWorker {
    /// Create a worker that decodes frames scaled to the preview bounds.
    pub fn new(max_width: u32, max_height: u32) -> Self {
        let (sender, receiver) = mpsc::channel::<DecodeRequest>();

        thread::spawn(move || {
            let _ = ffmpeg::init();
            let mut decoders: HashMap<PathBuf, DecoderEntry> = HashMap::new();
            let mut access_counter: u64 = 0;

            for request in receiver {
                let DecodeRequest {
                    path,
                    time_seconds,
                    mode,
                    respond_to,
                } = request;

                let image = match decoders.entry(path.clone()) {
                    Entry::Occupied(mut entry) => {
                        access_counter = access_counter.wrapping_add(1);
                        entry.get_mut().last_used = access_counter;
                        entry
                            .get_mut()
                            .decoder
                            .decode_frame_at_time(time_seconds, mode)
                    }
                    Entry::Vacant(entry) => match VideoDecoder::open(&path, max_width, max_height) {
                        Ok(mut decoder) => {
                            access_counter = access_counter.wrapping_add(1);
                            let image = decoder.decode_frame_at_time(time_seconds, mode);
                            entry.insert(DecoderEntry {
                                decoder,
                                last_used: access_counter,
                            });
                            image
                        }
                        Err(_) => None,
                    },
                };

                if decoders.len() > MAX_DECODERS {
                    evict_least_used(&mut decoders);
                }

                let _ = respond_to.send(image);
            }
        });

        Self { sender }
    }

    /// Decode a single frame at the requested timestamp (seconds).
    pub fn decode(&self, path: &Path, time_seconds: f64) -> Option<RgbaImage> {
        self.decode_with_mode(path, time_seconds, DecodeMode::Seek)
    }

    /// Decode a single frame using sequential decode when possible.
    pub fn decode_sequential(&self, path: &Path, time_seconds: f64) -> Option<RgbaImage> {
        self.decode_with_mode(path, time_seconds, DecodeMode::Sequential)
    }

    fn decode_with_mode(
        &self,
        path: &Path,
        time_seconds: f64,
        mode: DecodeMode,
    ) -> Option<RgbaImage> {
        let (respond_to, response) = mpsc::channel();
        let request = DecodeRequest {
            path: path.to_path_buf(),
            time_seconds,
            mode,
            respond_to,
        };

        self.sender.send(request).ok()?;
        response.recv().ok().flatten()
    }
}

struct DecoderEntry {
    decoder: VideoDecoder,
    last_used: u64,
}

fn evict_least_used(decoders: &mut HashMap<PathBuf, DecoderEntry>) {
    while decoders.len() > MAX_DECODERS {
        let mut oldest_key: Option<PathBuf> = None;
        let mut oldest_stamp: u64 = u64::MAX;
        for (key, entry) in decoders.iter() {
            if entry.last_used < oldest_stamp {
                oldest_stamp = entry.last_used;
                oldest_key = Some(key.clone());
            }
        }
        if let Some(key) = oldest_key {
            decoders.remove(&key);
        } else {
            break;
        }
    }
}

struct VideoDecoder {
    input: ffmpeg::format::context::Input,
    stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    scaler: ffmpeg::software::scaling::Context,
    time_base: ffmpeg::Rational,
    last_pts: Option<i64>,
    last_time_seconds: Option<f64>,
}

impl VideoDecoder {
    fn open(path: &Path, max_width: u32, max_height: u32) -> Result<Self, ffmpeg::Error> {
        let input = ffmpeg::format::input(path)?;
        let stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let stream_index = stream.index();
        let time_base = stream.time_base();

        let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = context.decoder().video()?;

        let src_width = decoder.width().max(1);
        let src_height = decoder.height().max(1);
        let (target_width, target_height) =
            fit_dimensions(src_width, src_height, max_width, max_height);

        let scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            src_width,
            src_height,
            ffmpeg::util::format::Pixel::RGBA,
            target_width,
            target_height,
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        Ok(Self {
            input,
            stream_index,
            decoder,
            scaler,
            time_base,
            last_pts: None,
            last_time_seconds: None,
        })
    }

    fn decode_frame_at_time(&mut self, time_seconds: f64, mode: DecodeMode) -> Option<RgbaImage> {
        match mode {
            DecodeMode::Seek => self.decode_with_seek(time_seconds),
            DecodeMode::Sequential => self.decode_sequential(time_seconds),
        }
    }

    fn decode_sequential(&mut self, time_seconds: f64) -> Option<RgbaImage> {
        let last_time = self.last_time_seconds.unwrap_or(f64::NEG_INFINITY);
        let delta = time_seconds - last_time;
        if self.last_pts.is_none() || delta < 0.0 || delta > MAX_SEQUENTIAL_JUMP_SECONDS {
            return self.decode_with_seek(time_seconds);
        }

        let target_pts = seconds_to_pts(time_seconds.max(0.0), self.time_base);
        let mut decoded = ffmpeg::util::frame::Video::empty();
        let mut rgba_frame = ffmpeg::util::frame::Video::empty();

        self.decode_forward(target_pts, &mut decoded, &mut rgba_frame)
            .or_else(|| self.decode_with_seek(time_seconds))
    }

    fn decode_with_seek(&mut self, time_seconds: f64) -> Option<RgbaImage> {
        let target_ts = (time_seconds.max(0.0) * AV_TIME_BASE as f64).round() as i64;
        if self.input.seek(target_ts, ..).is_err() {
            return None;
        }
        self.decoder.flush();

        let target_pts = seconds_to_pts(time_seconds.max(0.0), self.time_base);
        let mut decoded = ffmpeg::util::frame::Video::empty();
        let mut rgba_frame = ffmpeg::util::frame::Video::empty();
        self.decode_forward(target_pts, &mut decoded, &mut rgba_frame)
    }

    fn decode_forward(
        &mut self,
        target_pts: i64,
        decoded: &mut ffmpeg::util::frame::Video,
        rgba_frame: &mut ffmpeg::util::frame::Video,
    ) -> Option<RgbaImage> {
        let time_base = self.time_base;
        let decoder = &mut self.decoder;
        let scaler = &mut self.scaler;

        if let Some((frame, pts)) =
            receive_until_target(decoder, scaler, target_pts, decoded, rgba_frame)
        {
            self.last_pts = Some(pts);
            self.last_time_seconds = Some(pts_to_seconds(pts, time_base));
            return Some(frame);
        }

        let input = &mut self.input;
        for (stream, packet) in input.packets() {
            if stream.index() != self.stream_index {
                continue;
            }
            if decoder.send_packet(&packet).is_err() {
                continue;
            }
            if let Some((frame, pts)) =
                receive_until_target(decoder, scaler, target_pts, decoded, rgba_frame)
            {
                self.last_pts = Some(pts);
                self.last_time_seconds = Some(pts_to_seconds(pts, time_base));
                return Some(frame);
            }
        }

        None
    }
}

fn fit_dimensions(src_width: u32, src_height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let max_width = max_width.max(1);
    let max_height = max_height.max(1);

    if src_width <= max_width && src_height <= max_height {
        return (src_width, src_height);
    }

    let scale_w = max_width as f64 / src_width as f64;
    let scale_h = max_height as f64 / src_height as f64;
    let scale = scale_w.min(scale_h).max(0.01);

    let target_width = (src_width as f64 * scale).round().max(1.0) as u32;
    let target_height = (src_height as f64 * scale).round().max(1.0) as u32;

    (target_width, target_height)
}

fn seconds_to_pts(time_seconds: f64, time_base: ffmpeg::Rational) -> i64 {
    let numerator = time_base.numerator() as f64;
    let denominator = time_base.denominator() as f64;
    if numerator <= 0.0 || denominator <= 0.0 {
        return 0;
    }

    (time_seconds * denominator / numerator).round() as i64
}

fn pts_to_seconds(pts: i64, time_base: ffmpeg::Rational) -> f64 {
    let numerator = time_base.numerator() as f64;
    let denominator = time_base.denominator() as f64;
    if numerator <= 0.0 || denominator <= 0.0 {
        return 0.0;
    }

    pts as f64 * numerator / denominator
}

fn receive_until_target(
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut ffmpeg::software::scaling::Context,
    target_pts: i64,
    decoded: &mut ffmpeg::util::frame::Video,
    rgba_frame: &mut ffmpeg::util::frame::Video,
) -> Option<(RgbaImage, i64)> {
    while decoder.receive_frame(decoded).is_ok() {
        let frame_pts = decoded.timestamp().or(decoded.pts()).unwrap_or(0);
        if frame_pts < target_pts {
            continue;
        }

        if scaler.run(decoded, rgba_frame).is_err() {
            continue;
        }

        let image = frame_to_rgba(rgba_frame)?;
        return Some((image, frame_pts));
    }
    None
}

fn frame_to_rgba(frame: &ffmpeg::util::frame::Video) -> Option<RgbaImage> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    if width == 0 || height == 0 {
        return None;
    }

    let stride = frame.stride(0);
    let row_bytes = width * 4;
    if stride < row_bytes {
        return None;
    }

    let data = frame.data(0);
    let mut buffer = vec![0_u8; row_bytes * height];

    for y in 0..height {
        let src_offset = y * stride;
        let dst_offset = y * row_bytes;
        let src_slice = data.get(src_offset..src_offset + row_bytes)?;
        buffer[dst_offset..dst_offset + row_bytes].copy_from_slice(src_slice);
    }

    RgbaImage::from_vec(width as u32, height as u32, buffer)
}
