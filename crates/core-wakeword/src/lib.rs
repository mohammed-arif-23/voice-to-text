use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WakeWordState {
    Armed,
    WakeDetected,
    Listening,
    Processing,
    Review,
    Injecting,
}

pub trait WakeWordDetector: Send + Sync {
    fn feed_audio(&mut self, samples: &[f32]) -> Result<bool, String>;
    fn set_muted(&mut self, muted: bool);
    fn is_muted(&self) -> bool;
    fn get_state(&self) -> WakeWordState;
    fn set_state(&mut self, state: WakeWordState);
}

pub struct PorcupineWakeWordDetector {
    state: Mutex<WakeWordState>,
    muted: AtomicBool,
    sliding_window: Mutex<Vec<f32>>,
}

impl PorcupineWakeWordDetector {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WakeWordState::Armed),
            muted: AtomicBool::new(false),
            sliding_window: Mutex::new(Vec::with_capacity(16000)), // 1 sec window
        }
    }
}

impl WakeWordDetector for PorcupineWakeWordDetector {
    fn feed_audio(&mut self, samples: &[f32]) -> Result<bool, String> {
        if self.muted.load(Ordering::SeqCst) {
            return Ok(false);
        }

        let mut window = self.sliding_window.lock().unwrap();
        // Maintain a sliding window of exactly 16000 samples (1 second at 16kHz)
        window.extend_from_slice(samples);
        if window.len() > 16000 {
            let drain_len = window.len() - 16000;
            window.drain(0..drain_len);
        }

        // Check if we are currently Armed before detecting wake phrase
        let current_state = *self.state.lock().unwrap();
        if current_state != WakeWordState::Armed {
            return Ok(false);
        }

        // Mock energy detection to simulate wake-word detection when amplitude exceeds threshold (e.g. testing)
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / samples.len().max(1) as f32).sqrt();

        // If RMS exceeds threshold, we simulate a "Hey ScribeRx" wake-word trigger
        if rms > 0.85 {
            *self.state.lock().unwrap() = WakeWordState::WakeDetected;
            println!("[WakeWordDetector]: 'Hey ScribeRx' wake-word detected via RMS energy trigger.");
            return Ok(true);
        }

        Ok(false)
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted.store(muted, Ordering::SeqCst);
        if muted {
            println!("[WakeWordDetector]: Microphone input software muted.");
        } else {
            println!("[WakeWordDetector]: Microphone input software unmuted.");
        }
    }

    fn is_muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst)
    }

    fn get_state(&self) -> WakeWordState {
        *self.state.lock().unwrap()
    }

    fn set_state(&mut self, state: WakeWordState) {
        let mut guard = self.state.lock().unwrap();
        *guard = state;
        println!("[WakeWordDetector] State changed to: {:?}", state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake_word_state_transitions() {
        let mut detector = PorcupineWakeWordDetector::new();
        assert_eq!(detector.get_state(), WakeWordState::Armed);
        
        detector.set_state(WakeWordState::WakeDetected);
        assert_eq!(detector.get_state(), WakeWordState::WakeDetected);
    }

    #[test]
    fn test_mute_guard() {
        let mut detector = PorcupineWakeWordDetector::new();
        detector.set_muted(true);
        assert!(detector.is_muted());
        
        let res = detector.feed_audio(&[0.99; 100]);
        assert_eq!(res.unwrap(), false); // Muted inputs are ignored
    }
}
