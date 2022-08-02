use log::trace;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

struct GeoIPFilter;

impl Context for GeoIPFilter{}

impl HttpContext for GeoIPFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        for (name, value) in &self.get_http_request_headers() {
            trace!("In WASM -> {}: {}", name, value);
        }

        Action::Pause
    }
}
