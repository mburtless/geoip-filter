use std::time::{SystemTime};
use proxy_wasm as wasm;

pub trait Host {
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn inc(&self, m: u32);
    fn current_time(&self) -> SystemTime;
}

#[derive(Clone)]
pub struct WasmHost;

impl Host for WasmHost {
    fn debug(&self, msg: &str) {
        let _ = proxy_wasm::hostcalls::log(wasm::types::LogLevel::Debug, msg).unwrap();
    }
    fn info(&self, msg: &str) {
        let _ = proxy_wasm::hostcalls::log(wasm::types::LogLevel::Info, msg);
    }
    fn warn(&self, msg: &str) {
        let _ = proxy_wasm::hostcalls::log(wasm::types::LogLevel::Warn, msg);
    }
    fn error(&self, msg: &str) {
        let _ = proxy_wasm::hostcalls::log(wasm::types::LogLevel::Error, msg);
    }
    fn inc(&self, m: u32) {
        let _ = proxy_wasm::hostcalls::increment_metric(m, 1);
    }
    fn current_time(&self) -> SystemTime {
        // C code bindings will always resolve current time.
        proxy_wasm::hostcalls::get_current_time().unwrap()
    }
}
