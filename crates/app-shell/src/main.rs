use core_audio::{AudioRecorder, DummyAudioRecorder};
use core_hotkey::{DummyInjector, TextInjector};
use drug_match::{DrugMatcher, DummyDrugMatcher};
use stt_engine::{DummySttEngine, SttEngine};

#[tokio::main]
async fn main() {
    println!("ScribeRx Desktop Shell Initializing...");
    
    let mut audio = DummyAudioRecorder::new();
    let stt = DummySttEngine;
    let matcher = DummyDrugMatcher;
    let injector = DummyInjector;

    let _ = audio.start_recording();
    let pcm = audio.stop_recording().unwrap();
    let res = stt.transcribe(&pcm, 16000).await.unwrap();
    let corrected = matcher.match_text(&res.raw_text);
    
    println!("Formatted result: {}", corrected.formatted_text);
    let _ = injector.inject(&corrected.formatted_text, core_hotkey::InjectionStrategy::SendInput);
}
