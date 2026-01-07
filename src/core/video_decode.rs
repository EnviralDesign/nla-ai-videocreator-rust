use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use ffmpeg_next as ffmpeg;
use image::RgbaImage;

const AV_TIME_BASE: i64 = 1_000_000;

struct DecodeRequest {
    path: PathBuf,
    time_seconds: f64,
    respond_to: mpsc::Sender<Option<RgbaImage>>,
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
            let mut decoders: HashMap<PathBuf, VideoDecoder> = HashMap::new();

            for request in receiver {
                let DecodeRequest {
                    path,
                    time_seconds,
                    respond_to,
                } = request;

                let image = match decoders.entry(path.clone()) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().decode_frame_at_time(time_seconds)
                    }
                    Entry::Vacant(entry) => {
                        match VideoDecoder::open(&path, max_width, max_height) {
                            Ok(mut decoder) => {
                                let image = decoder.decode_frame_at_time(time_seconds);
                                entry.insert(decoder);
                                image
                            }
                            Err(_) => None,
                        }
                    }
                };

                let _ = respond_to.send(image);
            }
        });

        Self { sender }
    }

    /// Decode a single frame at the requested timestamp (seconds).
    pub fn decode(&self, path: &Path, time_seconds: f64) -> Option<RgbaImage> {
        let (respond_to, response) = mpsc::channel();
        let request = DecodeRequest {
            path: path.to_path_buf(),
            time_seconds,
            respond_to,
        };

        self.sender.send(request).ok()?;
        response.recv().ok().flatten()
    }
}

struct VideoDecoder {
    input: ffmpeg::format::context::Input,
    stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    scaler: ffmpeg::software::scaling::Context,
    time_base: ffmpeg::Rational,
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
        })
    }

    fn decode_frame_at_time(&mut self, time_seconds: f64) -> Option<RgbaImage> {
        let target_ts = (time_seconds.max(0.0) * AV_TIME_BASE as f64).round() as i64;
        if self.input.seek(target_ts, ..).is_err() {
            return None;
        }
        self.decoder.flush();

        let target_pts = seconds_to_pts(time_seconds.max(0.0), self.time_base);
        let mut decoded = ffmpeg::util::frame::Video::empty();
        let mut rgba_frame = ffmpeg::util::frame::Video::empty();

        for (stream, packet) in self.input.packets() {
            if stream.index() != self.stream_index {
                continue;
            }
            if self.decoder.send_packet(&packet).is_err() {
                continue;
            }

            while self.decoder.receive_frame(&mut decoded).is_ok() {
                if let Some(frame_pts) = decoded.timestamp().or(decoded.pts()) {
                    if frame_pts < target_pts {
                        continue;
                    }
                }

                if self.scaler.run(&decoded, &mut rgba_frame).is_err() {
                    continue;
                }

                return frame_to_rgba(&rgba_frame);
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
