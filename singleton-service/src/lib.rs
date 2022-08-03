use std::time::Duration;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use host::*;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(SingletonService{
            context_id,
            host: WasmHost{},
        })
    });
}

struct SingletonService<T> {
    context_id: u32,
    host: T,
}

impl <T: 'static + Host + Clone> RootContext for SingletonService<T> {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // tick immediately to obtain maxmind db
        //let _ = proxy_wasm::hostcalls::log(LogLevel::Warn, "VM instantiated");
        self.host.info("VM instantiated");
        self.set_tick_period(Duration::from_secs(5));
        true
    }
}

impl <T: Host + Clone> Context for SingletonService<T> {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(body) = self.get_http_call_response_body(0, _body_size) {
            if let Ok(body) = std::str::from_utf8(&body) {
                /*let _ = proxy_wasm::hostcalls::log(
                    LogLevel::Info,
                    format!("HTTP Call Response : {:?}", body).as_str(),
                );*/
                self.host.info(format!("HTTP Call Response : {:?}", body).as_str());
            }
        }
    }
}