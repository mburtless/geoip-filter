use std::net::IpAddr;
use std::time::Duration;
use log::{trace};
use proxy_wasm as wasm;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use common::*;
use maxminddb::Reader;
use std::rc::Rc;

#[derive(Clone)]
struct RootHandler<T> {
    host: T,

    // filter_config holds the JSON configuration set within the Envoy bootstrap config.
    //filter_config: config::FilterCfg,

    // When maxmind db is loaded for the first time, this will be true
    mmdb_ready: bool,

    mmdb_reader: Option<Rc<Reader<Bytes>>>,
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
            mmdb_ready: false,
            /*filter_config: config::FilterCfg {
                maxmind_url: "https://127.0.0.1/GeoLite2-Country.mmdb".to_string(),
            },*/
            mmdb_reader: None,
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
        if !self.mmdb_ready {
            self.host.info("mmdb not yet loaded into worker, attempting to load");
            // attempt to load mmdb from shared mem
            match self.get_shared_data(SHARED_MEMORY_KEY) {
                (Some(mmdb), _) => {
                    self.host.info("mmdb loaded from shared data");

                    // init reader from mmdb
                    match Reader::from_source(mmdb) {
                        Ok(r) => {
                            // put reader in rc so we can use it in each http context without cloning
                            self.mmdb_reader = Some(Rc::new(r));
                            self.host.info("mmdb reader initialized");
                        }
                        Err(e) => {
                            self.host.error(
                                format!("unable to init mmdb reader from mmdb in shared data: {:?}", e).as_str()
                            );
                        }
                    }

                    self.mmdb_ready = true;
                    self.set_tick_period(Duration::from_secs(60 * 30)); // Tick every 30 minutes.
                }
                (None, _) => {
                    self.host.warn("mmdb is missing from shared data");
                }
            }
        }
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        if self.mmdb_ready && self.mmdb_reader.is_some(){
            let r = self.mmdb_reader.as_ref().unwrap();
            Some(Box::new(GeoIPFilter{
                host: self.host.clone(),
                //filter_config: self.filter_config.clone(),
                mmdb_reader: Rc::clone(r),
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
    //filter_config: config::FilterCfg,
    mmdb_reader: Rc<Reader<Bytes>>,
}

impl<T: Host> Context for GeoIPFilter<T> {}

impl<T: Host> HttpContext for GeoIPFilter<T> {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        /*for (name, value) in &self.get_http_request_headers() {
            self.host.info(format!("In WASM -> {}: {}", name, value).as_str());
        }*/
        // TODO: account for real xff which could be a list
        match self.get_http_request_header("x-forwarded-for") {
            Some(v) => {
                let ip: IpAddr = v.parse().unwrap();
                let geo: maxminddb::geoip2::Country = self.mmdb_reader.lookup(ip).unwrap();
                let country = geo.country.unwrap().to_owned();
                self.host.info(format!("country resolved: {:#?}", country.iso_code).as_str());
                self.add_http_request_header(
                    "x-country-code",
                    country.iso_code.unwrap(),
                );
            }
            None => {
                self.host.warn("xff header not found in request");
            }
        }

        Action::Continue
    }
}
