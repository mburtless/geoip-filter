mod config;

use std::time::Duration;
use log::{trace};
use proxy_wasm as wasm;
use proxy_wasm::hostcalls::get_shared_data;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use common::*;



#[derive(Clone)]
struct RootHandler<T> {
    host: T,

    // filter_config holds the JSON configuration set within the Envoy bootstrap config.
    filter_config: config::FilterCfg,

    // When maxmind db is loaded for the first time, this will be true
    maxmind_db_ready: bool,
}

// This handler is used for requests when the filter is not ready. It's a no-op.
struct PassthroughHTTPHandler;
impl HttpContext for PassthroughHTTPHandler {}
impl Context for PassthroughHTTPHandler {}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(wasm::types::LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(RootHandler {
            host: WasmHost {},
            maxmind_db_ready: false,
            filter_config: config::FilterCfg {
                maxmind_url: "https://127.0.0.1/GeoLite2-Country.mmdb".to_string(),
            },
        })
    });
}

impl<T: 'static + Host + Clone> RootContext for RootHandler<T> {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // tick immediately to obtain maxmind db
        self.set_tick_period(Duration::from_millis(1000));
        true
    }

    fn on_tick(&mut self) {
        if !self.maxmind_db_ready {
            self.host.info("mmdb not yet loaded into worker, attempting to load");
            // attempt to load mmdb from shared mem
            match self.get_shared_data(SHARED_MEMORY_KEY) {
                (Some(_cache), _) => {
                    self.host.info("mmdb loaded from shared memory");
                    self.maxmind_db_ready = true;
                    self.set_tick_period(Duration::from_secs(60 * 30)); // Tick every 30 minutes.
                }
                (None, _) => {
                    self.host.warn("mmdb is missing from cache");
                }
            }
        }
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        if self.maxmind_db_ready {
            Some(Box::new(GeoIPFilter{
                host: self.host.clone(),
                filter_config: self.filter_config.clone(),
            }))
        } else {
            self.host.warn("filter not ready so request passed through");
            Some(Box::new(PassthroughHTTPHandler))
        }
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl<T: Host + Clone> Context for RootHandler<T> {}

struct GeoIPFilter<T> {
    host: T,
    filter_config: config::FilterCfg,
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
