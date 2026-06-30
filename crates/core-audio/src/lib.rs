use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering, AtomicBool};

pub trait AudioRecorder: Send + Sync {
    fn start_recording(&mut self) -> Result<(), String>;
    fn stop_recording(&mut self) -> Result<Vec<f32>, String>;
    fn get_current_rms_level(&self) -> f32;
}

pub struct DummyAudioRecorder {
    recording: bool,
}

impl DummyAudioRecorder {
    pub fn new() -> Self {
        Self { recording: false }
    }
}

impl AudioRecorder for DummyAudioRecorder {
    fn start_recording(&mut self) -> Result<(), String> {
        self.recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        self.recording = false;
        Ok(vec![0.0; 16000]) // 1 sec dummy PCM
    }

    fn get_current_rms_level(&self) -> f32 {
        if self.recording { 0.42 } else { 0.0 }
    }
}

#[cfg(target_os = "windows")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(target_os = "windows")]
pub struct StreamWrapper(cpal::Stream);
#[cfg(target_os = "windows")]
unsafe impl Send for StreamWrapper {}
#[cfg(target_os = "windows")]
unsafe impl Sync for StreamWrapper {}

#[cfg(target_os = "windows")]
pub struct CpalAudioRecorder {
    stream: Option<StreamWrapper>,
    buffer: Arc<Mutex<Vec<f32>>>,
    rms_level: Arc<AtomicU32>,
    input_sample_rate: u32,
    input_channels: u16,
    recording: Arc<AtomicBool>,
}

#[cfg(target_os = "windows")]
impl CpalAudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            rms_level: Arc::new(AtomicU32::new(0f32.to_bits())),
            input_sample_rate: 44100,
            input_channels: 1,
            recording: Arc::new(AtomicBool::new(false)),
        }
    }

    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }
        let ratio = to_rate as f64 / from_rate as f64;
        let new_len = (samples.len() as f64 * ratio).floor() as usize;
        let mut resampled = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let pos = i as f64 / ratio;
            let low = pos.floor() as usize;
            let high = pos.ceil() as usize;
            if high >= samples.len() {
                resampled.push(samples[low]);
                continue;
            }
            let weight = pos - low as f64;
            let val = samples[low] * (1.0 - weight as f32) + samples[high] * weight as f32;
            resampled.push(val);
        }
        resampled
    }
}

#[cfg(target_os = "windows")]
impl AudioRecorder for CpalAudioRecorder {
    fn start_recording(&mut self) -> Result<(), String> {
        {
            let mut buf = self.buffer.lock().unwrap();
            buf.clear();
        }
        self.recording.store(true, Ordering::SeqCst);

        if self.stream.is_none() {
            let host = cpal::default_host();
            let device = host.default_input_device()
                .ok_or_else(|| "No default input audio device found".to_string())?;

            let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
            println!("Using default audio input device: \"{}\"", name);

            let config = device.default_input_config()
                .map_err(|e| format!("Failed to get default input config: {}", e))?;

            self.input_sample_rate = config.sample_rate().0;
            self.input_channels = config.channels();

            let buffer = Arc::clone(&self.buffer);
            let rms_level = Arc::clone(&self.rms_level);
            let recording = Arc::clone(&self.recording);

            let err_fn = |err| eprintln!("An error occurred on the audio input stream: {}", err);

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                            let rms = Self::calculate_rms(data);
                            rms_level.store(rms.to_bits(), Ordering::SeqCst);
                            if recording.load(Ordering::SeqCst) {
                                if let Ok(mut buf) = buffer.lock() {
                                    buf.extend_from_slice(data);
                                }
                            }
                        },
                        err_fn,
                        None
                    )
                }
                cpal::SampleFormat::I16 => {
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            let f32_data: Vec<f32> = data.iter().map(|&x| x as f32 / 32768.0).collect();
                            let rms = Self::calculate_rms(&f32_data);
                            rms_level.store(rms.to_bits(), Ordering::SeqCst);
                            if recording.load(Ordering::SeqCst) {
                                if let Ok(mut buf) = buffer.lock() {
                                    buf.extend(f32_data);
                                }
                            }
                        },
                        err_fn,
                        None
                    )
                }
                cpal::SampleFormat::U16 => {
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &_| {
                            let f32_data: Vec<f32> = data.iter().map(|&x| (x as f32 - 32768.0) / 32768.0).collect();
                            let rms = Self::calculate_rms(&f32_data);
                            rms_level.store(rms.to_bits(), Ordering::SeqCst);
                            if recording.load(Ordering::SeqCst) {
                                if let Ok(mut buf) = buffer.lock() {
                                    buf.extend(f32_data);
                                }
                            }
                        },
                        err_fn,
                        None
                    )
                }
                _ => return Err("Unsupported sample format".to_string()),
            }.map_err(|e| format!("Failed to build input stream: {}", e))?;

            stream.play().map_err(|e| format!("Failed to play input stream: {}", e))?;
            self.stream = Some(StreamWrapper(stream));
        }

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        self.recording.store(false, Ordering::SeqCst);
        self.rms_level.store(0f32.to_bits(), Ordering::SeqCst);

        let raw_samples = {
            let mut buf = self.buffer.lock().unwrap();
            let samples = buf.clone();
            buf.clear();
            samples
        };

        if raw_samples.is_empty() {
            return Ok(Vec::new());
        }

        let mono_samples = if self.input_channels > 1 {
            let mut mono = Vec::with_capacity(raw_samples.len() / self.input_channels as usize);
            let chunk_size = self.input_channels as usize;
            for chunk in raw_samples.chunks_exact(chunk_size) {
                let sum: f32 = chunk.iter().sum();
                mono.push(sum / self.input_channels as f32);
            }
            mono
        } else {
            raw_samples
        };

        let target_sample_rate = 16000;
        let resampled = Self::resample(&mono_samples, self.input_sample_rate, target_sample_rate);

        Ok(resampled)
    }

    fn get_current_rms_level(&self) -> f32 {
        f32::from_bits(self.rms_level.load(Ordering::SeqCst))
    }
}

#[cfg(not(target_os = "windows"))]
pub struct CpalAudioRecorder {
    recording: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
    rms_level: Arc<AtomicU32>,
}

#[cfg(not(target_os = "windows"))]
impl CpalAudioRecorder {
    pub fn new() -> Self {
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            rms_level: Arc::new(AtomicU32::new(0f32.to_bits())),
        }
    }

    pub fn feed_mock_audio(&self, samples: &[f32]) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.extend_from_slice(samples);
        }
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / samples.len().max(1) as f32).sqrt();
        self.rms_level.store(rms.to_bits(), Ordering::SeqCst);
    }
}

#[cfg(not(target_os = "windows"))]
impl AudioRecorder for CpalAudioRecorder {
    fn start_recording(&mut self) -> Result<(), String> {
        self.recording.store(true, Ordering::SeqCst);
        self.rms_level.store(0.15f32.to_bits(), Ordering::SeqCst);
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        self.recording.store(false, Ordering::SeqCst);
        self.rms_level.store(0f32.to_bits(), Ordering::SeqCst);
        let buf = self.buffer.lock().unwrap();
        if buf.is_empty() {
            Ok(vec![0.05; 16000]) // 1 sec of dummy speech samples
        } else {
            Ok(buf.clone())
        }
    }

    fn get_current_rms_level(&self) -> f32 {
        f32::from_bits(self.rms_level.load(Ordering::SeqCst))
    }
}
