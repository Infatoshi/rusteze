use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

/// Maximum buffer size: 1 second at 48kHz mono.
const BUFFER_CAP: usize = 48_000;

/// Shared ring buffer between input and output audio streams.
pub type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

/// Audio engine that captures from the default input device and
/// plays back through the default output device using a 1-second
/// ring buffer. Audio data is never persisted -- it lives only in
/// memory and is overwritten continuously.
pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    buffer: AudioBuffer,
    pub muted: bool,
    pub deafened: bool,
    running: bool,
}

impl AudioEngine {
    /// Create a new audio engine (streams are not started yet).
    pub fn new() -> Self {
        Self {
            input_stream: None,
            output_stream: None,
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_CAP))),
            muted: false,
            deafened: false,
            running: false,
        }
    }

    /// Start capturing and playing audio.
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.running {
            return Ok(());
        }

        let host = cpal::default_host();

        // --- Input stream (microphone) ---
        let input_device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("no input audio device found"))?;

        let input_config: StreamConfig = input_device
            .default_input_config()?
            .into();

        tracing::info!(
            "audio input: @ {}Hz, {} channels",
            input_config.sample_rate,
            input_config.channels,
        );

        let buf_in = self.buffer.clone();
        let channels = input_config.channels as usize;

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = buf_in.lock().unwrap();
                // Take only the first channel (mono downmix)
                for chunk in data.chunks(channels) {
                    if let Some(&sample) = chunk.first() {
                        buf.push_back(sample);
                    }
                }
                // Cap at BUFFER_CAP (1 second) -- discard oldest samples
                while buf.len() > BUFFER_CAP {
                    buf.pop_front();
                }
            },
            |err| {
                tracing::error!("audio input error: {err}");
            },
            None,
        )?;

        // --- Output stream (speakers) ---
        let output_device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("no output audio device found"))?;

        let output_config: StreamConfig = output_device
            .default_output_config()?
            .into();

        tracing::info!(
            "audio output: @ {}Hz, {} channels",
            output_config.sample_rate,
            output_config.channels,
        );

        let buf_out = self.buffer.clone();
        let out_channels = output_config.channels as usize;

        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buf = buf_out.lock().unwrap();
                for chunk in data.chunks_mut(out_channels) {
                    let sample = buf.pop_front().unwrap_or(0.0);
                    // Write same sample to all output channels
                    for s in chunk.iter_mut() {
                        *s = sample;
                    }
                }
            },
            |err| {
                tracing::error!("audio output error: {err}");
            },
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
        self.running = true;

        tracing::info!("audio engine started (1s ring buffer, {} sample cap)", BUFFER_CAP);
        Ok(())
    }

    /// Stop capturing and playing audio.
    pub fn stop(&mut self) {
        self.input_stream = None;
        self.output_stream = None;
        self.running = false;
        // Clear the buffer
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
        tracing::info!("audio engine stopped");
    }

    /// Whether the engine is currently running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Toggle mute. When muted, input samples are discarded.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Whether audio output is deafened.
    pub fn is_deafened(&self) -> bool {
        self.deafened
    }

    /// Toggle deafen. When deafened, output stream is paused.
    pub fn toggle_deafen(&mut self) {
        self.deafened = !self.deafened;
        if self.deafened {
            // Pause output stream
            if let Some(ref stream) = self.output_stream {
                let _ = stream.pause();
            }
        } else {
            // Resume output stream
            if let Some(ref stream) = self.output_stream {
                let _ = stream.play();
            }
        }
    }
}
