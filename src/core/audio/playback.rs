//! Audio playback engine (cpal mixer + audio clock).

#![allow(dead_code)]

use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Clone)]
pub struct PlaybackItem {
    pub samples: Arc<Vec<f32>>,
    pub start_frame: u64,
    pub channels: u16,
}

impl PlaybackItem {
    pub fn frames(&self) -> u64 {
        let channels = self.channels.max(1) as usize;
        (self.samples.len() / channels) as u64
    }

    pub fn end_frame(&self) -> u64 {
        self.start_frame + self.frames()
    }
}

pub struct AudioPlaybackEngine {
    stream: cpal::Stream,
    items: Arc<Mutex<Vec<PlaybackItem>>>,
    playing: Arc<AtomicBool>,
    playhead_frames: Arc<AtomicU64>,
    sample_rate: u32,
    channels: u16,
}

impl AudioPlaybackEngine {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device found.".to_string())?;
        let config = select_output_config(&device)?;
        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        let items = Arc::new(Mutex::new(Vec::<PlaybackItem>::new()));
        let playing = Arc::new(AtomicBool::new(false));
        let playhead_frames = Arc::new(AtomicU64::new(0));

        let items_for_cb = Arc::clone(&items);
        let playing_for_cb = Arc::clone(&playing);
        let playhead_for_cb = Arc::clone(&playhead_frames);
        let channels_for_cb = channels;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if !playing_for_cb.load(Ordering::Relaxed) {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    let frames = data.len() / channels_for_cb as usize;
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }

                    let start_frame = playhead_for_cb.load(Ordering::Relaxed);
                    let end_frame = start_frame + frames as u64;

                    if let Ok(items) = items_for_cb.lock() {
                        for item in items.iter() {
                            if item.channels != channels_for_cb {
                                continue;
                            }
                            let item_start = item.start_frame;
                            let item_end = item.end_frame();
                            if item_end <= start_frame || item_start >= end_frame {
                                continue;
                            }

                            let overlap_start = start_frame.max(item_start);
                            let overlap_end = end_frame.min(item_end);
                            let overlap_frames = (overlap_end - overlap_start) as usize;
                            let buffer_offset =
                                (overlap_start - start_frame) as usize * channels_for_cb as usize;
                            let item_offset =
                                (overlap_start - item_start) as usize * channels_for_cb as usize;

                            let slice_end = item_offset + overlap_frames * channels_for_cb as usize;
                            if slice_end > item.samples.len() {
                                continue;
                            }

                            for i in 0..(overlap_frames * channels_for_cb as usize) {
                                data[buffer_offset + i] += item.samples[item_offset + i];
                            }
                        }
                    }

                    playhead_for_cb.store(end_frame, Ordering::Relaxed);
                },
                move |err| {
                    println!("Audio output error: {}", err);
                },
                None,
            )
            .map_err(|err| err.to_string())?;

        stream.play().map_err(|err| err.to_string())?;

        Ok(Self {
            stream,
            items,
            playing,
            playhead_frames,
            sample_rate,
            channels,
        })
    }

    pub fn set_items(&self, items: Vec<PlaybackItem>) {
        if let Ok(mut guard) = self.items.lock() {
            *guard = items;
        }
    }

    pub fn play(&self) {
        self.playing.store(true, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.playing.store(false, Ordering::Relaxed);
    }

    pub fn seek_seconds(&self, time_seconds: f64) {
        let frame = (time_seconds.max(0.0) * self.sample_rate as f64).round() as u64;
        self.playhead_frames.store(frame, Ordering::Relaxed);
    }

    pub fn playhead_seconds(&self) -> f64 {
        self.playhead_frames.load(Ordering::Relaxed) as f64 / self.sample_rate as f64
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }
}

fn select_output_config(device: &cpal::Device) -> Result<cpal::StreamConfig, String> {
    let configs: Vec<_> = device
        .supported_output_configs()
        .map_err(|err| err.to_string())?
        .filter(|config| config.sample_format() == cpal::SampleFormat::F32)
        .collect();

    let target_rate = cpal::SampleRate(48_000);
    if let Some(config) = configs.iter().find(|config| {
        config.min_sample_rate() <= target_rate && config.max_sample_rate() >= target_rate
    }) {
        return Ok(config.with_sample_rate(target_rate).config());
    }

    let default_config = device
        .default_output_config()
        .map_err(|err| err.to_string())?;
    if default_config.sample_format() != cpal::SampleFormat::F32 {
        return Err("Default output device does not support f32 sample format.".to_string());
    }
    Ok(default_config.config())
}
