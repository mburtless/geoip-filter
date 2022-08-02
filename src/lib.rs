mod os;

use log::trace;
use os::Host;
use proxy_wasm as wasm;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::{SystemTime};

#[derive(Clone)]
struct WasmHost;

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


#[derive(Clone)]
struct RootHandler<T> {
    host: T,
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(RootHandler {
            host: WasmHost {}
        })
    });
}

impl<T: 'static + Host + Clone> RootContext for RootHandler<T> {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // tick immediately to obtain maxmind db
        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
       Some(Box::new(GeoIPFilter{
           host: self.host.clone()
       }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl<T: Host + Clone> Context for RootHandler<T> {}

struct GeoIPFilter<T> {
    host: T,
}

impl<T: Host> Context for GeoIPFilter<T> {}

impl<T: Host> HttpContext for GeoIPFilter<T> {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        for (name, value) in &self.get_http_request_headers() {
            trace!("In WASM -> {}: {}", name, value);
        }

        Action::Continue
    }
}
