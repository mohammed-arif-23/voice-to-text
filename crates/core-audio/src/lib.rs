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
