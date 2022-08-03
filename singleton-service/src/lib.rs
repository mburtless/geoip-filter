mod config;

use std::time::Duration;
use proxy_wasm::hostcalls::set_shared_data;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use common::*;
use url::Url;

struct SingletonService<T> {
    host: T,

    // filter_config holds the JSON configuration set within the Envoy bootstrap config.
    filter_config: config::FilterCfg,

    // When maxmind db is loaded for the first time, this will be true
    mmdb_ready: bool,
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(SingletonService{
            host: WasmHost{},
            mmdb_ready: false,
            filter_config: config::FilterCfg {
                mmdb_url: "https://127.0.0.1/GeoLite2-Country.mmdb".to_string(),
                mmdb_cluster: "mmdb".to_string(),
            },
        })
    });
}

impl <T: 'static + Host + Clone> RootContext for SingletonService<T> {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        // tick immediately to obtain maxmind db
        self.set_tick_period(Duration::from_millis(1000));
        //let _ = proxy_wasm::hostcalls::log(LogLevel::Warn, "VM instantiated");
        self.host.info("VM instantiated");

        // Initialize shared memory to store maxmind db
        /*if let Err(e) = set_shared_data(
            SHARED_MEMORY_KEY,
            Some(&SHARED_MEMORY_INITIAL_SIZE.to_be_bytes()),
            None,
        ) {
            self.host.error(format!("Initializing shared memory key failed: {:?}", e).as_str());
            return false;
        }*/

        true
    }

    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.host.info("loaded filter config");

            self.filter_config = match config::FilterCfg::new(config_bytes) {
                Err(e) => {
                    self.host
                        .error(format!("Error parsing configuration json: {:?}", e).as_str());
                    return false;
                }
                Ok(cfg) => cfg,
            };
        } else {
            self.host.error("No filter config found - this should be set within the Envoy config");
            return false;
        }
        true
    }

    fn on_tick(&mut self) {
        self.set_tick_period(Duration::from_secs(60*60)); // Tick every hour
        self.request_mmdb();
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
            if !body.is_empty() {
                self.host.info("HTTP call response received");
                // init reader with body
                //let reader = maxminddb::Reader::from_source(body);

                // persist to shared data
                if let Err(e) = set_shared_data(
                    SHARED_MEMORY_KEY,
                    Some(&*body),
                    None,
                ) {
                    self.host.error(format!("persisting mmdb to shared memory failed: {:?}", e).as_str());
                }
                return;
            }
            self.host.error("HTTP call failed");
        }
    }
}

impl <T: Host + Clone> SingletonService<T> {
    fn request_mmdb(&mut self) {
        match Url::parse(self.filter_config.mmdb_url.as_str()) {
            Ok(url) => {
                match self.dispatch_http_call(
                    self.filter_config.mmdb_cluster.as_str(),
                    vec![
                        (":method", "GET"),
                        (":path", url.path()),
                        (":cache-control", "no-cache"),
                        (":authority", url.host_str().unwrap_or("mmdb")),
                    ],
                    None,
                    vec![],
                    Duration::from_secs(5),
                ) {
                    Ok(_s) => self.host.info("fetched mmdb"),
                    Err(err) => self.host.error(format!("failed to fetch mmdb via upstream {}{}: {:?}", self.filter_config.mmdb_cluster, url.path(), err).as_str()),
                }

            }
            Err(_) => {
                self.host.error(
                    format!("error while parsing the mmdb URL: {}", self.filter_config.mmdb_url).as_str()
                );
            }
        }
    }
}