#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionStrategy {
    SendInput,
    ClipboardPaste,
    UiAutomation,
}

pub trait TextInjector: Send + Sync {
    fn inject(&self, text: &str, strategy: InjectionStrategy) -> Result<(), String>;
    fn capture_caret_position(&self) -> Option<(i32, i32)>;
}

pub struct DummyInjector;

impl TextInjector for DummyInjector {
    fn inject(&self, text: &str, _strategy: InjectionStrategy) -> Result<(), String> {
        println!("[Mock Injection]: {}", text);
        Ok(())
    }

    fn capture_caret_position(&self) -> Option<(i32, i32)> {
        Some((100, 100))
    }
}
