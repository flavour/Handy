use std::{
    io::Error,
    sync::{mpsc, Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Sample, SizedSample,
};

use crate::audio_toolkit::{
    audio::{AudioVisualiser, FrameResampler},
    constants,
    vad::{self, VadFrame},
    VoiceActivityDetector,
};

type SpeechFrameCallback = Arc<dyn Fn(&[f32]) + Send + Sync + 'static>;

enum Cmd {
    Start,
    Stop(mpsc::Sender<Vec<f32>>),
    Shutdown,
}

pub struct AudioRecorder {
    device: Option<Device>,
    cmd_tx: Option<mpsc::Sender<Cmd>>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
    vad: Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
    level_cb: Option<Arc<dyn Fn(Vec<f32>) + Send + Sync + 'static>>,
    // Speech frame tap: invoked for each VAD-gated 30ms speech frame
    speech_frame_cb: Option<SpeechFrameCallback>,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AudioRecorder {
            device: None,
            cmd_tx: None,
            worker_handle: None,
            vad: None,
            level_cb: None,
            speech_frame_cb: None,
        })
    }

    pub fn with_vad(mut self, vad: Box<dyn VoiceActivityDetector>) -> Self {
        self.vad = Some(Arc::new(Mutex::new(vad)));
        self
    }

    pub fn with_speech_frame_callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(&[f32]) + Send + Sync + 'static,
    {
        self.speech_frame_cb = Some(Arc::new(cb));
        self
    }

    pub fn with_level_callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(Vec<f32>) + Send + Sync + 'static,
    {
        self.level_cb = Some(Arc::new(cb));
        self
    }

    pub fn open(&mut self, device: Option<Device>) -> Result<(), Box<dyn std::error::Error>> {
        if self.worker_handle.is_some() {
            return Ok(()); // already open
        }

        let (sample_tx, sample_rx) = mpsc::channel::<Vec<f32>>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();

        let host = crate::audio_toolkit::get_cpal_host();
        let device = match device {
            Some(dev) => dev,
            None => {
                // On Linux (especially under PipeWire), CPAL's ALSA host may expose built-in
                // devices like "pipewire" or "pulse" that can be more robust than selecting a
                // specific hardware device directly.
                #[cfg(target_os = "linux")]
                {
                    if let Ok(mut devices) = host.input_devices() {
                        if let Some(dev) =
                            devices.find(|d| d.name().ok().is_some_and(|n| n == "pipewire"))
                        {
                            log::info!("Audio: using built-in ALSA device: pipewire");
                            dev
                        } else if let Ok(mut devices) = host.input_devices() {
                            if let Some(dev) =
                                devices.find(|d| d.name().ok().is_some_and(|n| n == "pulse"))
                            {
                                log::info!("Audio: using built-in ALSA device: pulse");
                                dev
                            } else {
                                host.default_input_device().ok_or_else(|| {
                                    Error::new(
                                        std::io::ErrorKind::NotFound,
                                        "No input device found",
                                    )
                                })?
                            }
                        } else {
                            host.default_input_device().ok_or_else(|| {
                                Error::new(std::io::ErrorKind::NotFound, "No input device found")
                            })?
                        }
                    } else {
                        host.default_input_device().ok_or_else(|| {
                            Error::new(std::io::ErrorKind::NotFound, "No input device found")
                        })?
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    host.default_input_device().ok_or_else(|| {
                        Error::new(std::io::ErrorKind::NotFound, "No input device found")
                    })?
                }
            }
        };

        let thread_device = device.clone();
        let vad = self.vad.clone();
        // Move the optional level callback into the worker thread
        let level_cb = self.level_cb.clone();
        let speech_cb = self.speech_frame_cb.clone();
        if speech_cb.is_some() {
            log::info!("Recorder: speech callback is present before spawn");
        } else {
            log::info!("Recorder: speech callback is NOT present before spawn");
        }

        let worker = std::thread::spawn(move || {
            let config = AudioRecorder::get_preferred_config(&thread_device)
                .expect("failed to fetch preferred config");

            let sample_rate = config.sample_rate().0;
            let channels = config.channels() as usize;

            log::info!(
                "Using device: {:?}\nSample rate: {}\nChannels: {}\nFormat: {:?}",
                thread_device.name(),
                sample_rate,
                channels,
                config.sample_format()
            );

            let stream = match config.sample_format() {
                cpal::SampleFormat::U8 => {
                    AudioRecorder::build_stream::<u8>(&thread_device, &config, sample_tx, channels)
                        .unwrap()
                }
                cpal::SampleFormat::I8 => {
                    AudioRecorder::build_stream::<i8>(&thread_device, &config, sample_tx, channels)
                        .unwrap()
                }
                cpal::SampleFormat::I16 => {
                    AudioRecorder::build_stream::<i16>(&thread_device, &config, sample_tx, channels)
                        .unwrap()
                }
                cpal::SampleFormat::I32 => {
                    AudioRecorder::build_stream::<i32>(&thread_device, &config, sample_tx, channels)
                        .unwrap()
                }
                cpal::SampleFormat::F32 => {
                    AudioRecorder::build_stream::<f32>(&thread_device, &config, sample_tx, channels)
                        .unwrap()
                }
                _ => panic!("unsupported sample format"),
            };

            stream.play().expect("failed to start stream");

            // keep the stream alive while we process samples
            run_consumer(sample_rate, vad, sample_rx, cmd_rx, level_cb, speech_cb);
            // stream is dropped here, after run_consumer returns
        });

        self.device = Some(device);
        self.cmd_tx = Some(cmd_tx);
        self.worker_handle = Some(worker);

        Ok(())
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = &self.cmd_tx {
            tx.send(Cmd::Start)?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let (resp_tx, resp_rx) = mpsc::channel();
        if let Some(tx) = &self.cmd_tx {
            tx.send(Cmd::Stop(resp_tx))?;
        }
        Ok(resp_rx.recv()?) // wait for the samples
    }

    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(Cmd::Shutdown);
        }
        if let Some(h) = self.worker_handle.take() {
            let _ = h.join();
        }
        self.device = None;
        Ok(())
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        sample_tx: mpsc::Sender<Vec<f32>>,
        channels: usize,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: Sample + SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let mut output_buffer = Vec::new();

        let stream_cb = move |data: &[T], _: &cpal::InputCallbackInfo| {
            output_buffer.clear();

            if channels == 1 {
                // Direct conversion without intermediate Vec
                output_buffer.extend(data.iter().map(|&sample| sample.to_sample::<f32>()));
            } else {
                // Convert to mono directly
                let frame_count = data.len() / channels;
                output_buffer.reserve(frame_count);

                for frame in data.chunks_exact(channels) {
                    let mono_sample = frame
                        .iter()
                        .map(|&sample| sample.to_sample::<f32>())
                        .sum::<f32>()
                        / channels as f32;
                    output_buffer.push(mono_sample);
                }
            }

            if sample_tx.send(output_buffer.clone()).is_err() {
                log::error!("Failed to send samples");
            }
        };

        device.build_input_stream(
            &config.clone().into(),
            stream_cb,
            |err| log::error!("Stream error: {}", err),
            None,
        )
    }

    fn get_preferred_config(
        device: &cpal::Device,
    ) -> Result<cpal::SupportedStreamConfig, Box<dyn std::error::Error>> {
        // Prefer the device default config and resample to 16kHz in the consumer.
        // Some devices/drivers claim to support 16kHz but error at runtime (e.g. ALSA POLLERR).
        let default_config = device.default_input_config()?;

        // Optional: try to open the stream at 16kHz if explicitly requested.
        // This is useful for debugging and for devices that truly support native 16kHz.
        if std::env::var_os("HANDY_AUDIO_TRY_16K").is_none() {
            return Ok(default_config);
        }

        let supported_configs = device.supported_input_configs()?;
        let mut best_config: Option<cpal::SupportedStreamConfigRange> = None;

        for config_range in supported_configs {
            if config_range.min_sample_rate().0 <= constants::WHISPER_SAMPLE_RATE
                && config_range.max_sample_rate().0 >= constants::WHISPER_SAMPLE_RATE
            {
                match best_config {
                    None => best_config = Some(config_range),
                    Some(ref current) => {
                        // Prioritize F32 > I16 > I32 > others
                        let score = |fmt: cpal::SampleFormat| match fmt {
                            cpal::SampleFormat::F32 => 4,
                            cpal::SampleFormat::I16 => 3,
                            cpal::SampleFormat::I32 => 2,
                            _ => 1,
                        };

                        if score(config_range.sample_format()) > score(current.sample_format()) {
                            best_config = Some(config_range);
                        }
                    }
                }
            }
        }

        if let Some(config) = best_config {
            return Ok(config.with_sample_rate(cpal::SampleRate(constants::WHISPER_SAMPLE_RATE)));
        }

        Ok(default_config)
    }
}

fn run_consumer(
    in_sample_rate: u32,
    vad: Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
    sample_rx: mpsc::Receiver<Vec<f32>>,
    cmd_rx: mpsc::Receiver<Cmd>,
    level_cb: Option<Arc<dyn Fn(Vec<f32>) + Send + Sync + 'static>>,
    speech_cb: Option<SpeechFrameCallback>,
) {
    if speech_cb.is_some() {
        log::info!("Recorder consumer: speech_cb present");
    } else {
        log::info!("Recorder consumer: speech_cb absent");
    }
    let mut frame_resampler = FrameResampler::new(
        in_sample_rate as usize,
        constants::WHISPER_SAMPLE_RATE as usize,
        std::time::Duration::from_millis(30),
    );

    let mut processed_samples = Vec::<f32>::new();
    let mut recording = false;

    // ---------- spectrum visualisation setup ---------------------------- //
    const BUCKETS: usize = 16;
    const WINDOW_SIZE: usize = 512;
    let mut visualizer = AudioVisualiser::new(
        in_sample_rate,
        WINDOW_SIZE,
        BUCKETS,
        400.0,  // vocal_min_hz
        4000.0, // vocal_max_hz
    );

    fn handle_frame(
        samples: &[f32],
        recording: bool,
        vad: &Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
        out_buf: &mut Vec<f32>,
        speech_cb: &Option<SpeechFrameCallback>,
    ) {
        if let Some(vad_arc) = vad {
            let mut det = vad_arc.lock().unwrap();
            match det.push_frame(samples).unwrap_or(VadFrame::Speech(samples)) {
                VadFrame::Speech(buf) => {
                    log::info!("VAD: Speech frame len={}", buf.len());
                    // Only run wake-word callbacks when NOT recording to avoid
                    // heavy inference during recording/stop flush.
                    if !recording {
                        if let Some(cb) = speech_cb {
                            log::info!("Recorder: invoking speech_cb");
                            cb(buf);
                        }
                    }
                    if recording {
                        out_buf.extend_from_slice(buf)
                    }
                }
                VadFrame::Noise => {
                    // log::info!("No speech");
                }
            }
        } else {
            // No VAD: do NOT forward to wake-word callback to avoid false triggers.
            // We still buffer if recording.
            if recording {
                out_buf.extend_from_slice(samples);
            }
        }
    }

    while let Ok(raw) = sample_rx.recv() {
        // ---------- spectrum processing ---------------------------------- //
        if let Some(buckets) = visualizer.feed(&raw) {
            if let Some(cb) = &level_cb {
                cb(buckets);
            }
        }

        // ---------- existing pipeline ------------------------------------ //
        frame_resampler.push(&raw, &mut |frame: &[f32]| {
            handle_frame(frame, recording, &vad, &mut processed_samples, &speech_cb)
        });

        // non-blocking check for a command
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Cmd::Start => {
                    processed_samples.clear();
                    recording = true;
                    visualizer.reset(); // Reset visualization buffer
                    if let Some(v) = &vad {
                        v.lock().unwrap().reset();
                    }
                }
                Cmd::Stop(reply_tx) => {
                    recording = false;

                    frame_resampler.finish(&mut |frame: &[f32]| {
                        // we still want to process the last few frames
                        handle_frame(frame, true, &vad, &mut processed_samples, &speech_cb)
                    });

                    let _ = reply_tx.send(std::mem::take(&mut processed_samples));
                }
                Cmd::Shutdown => return,
            }
        }
    }
}
