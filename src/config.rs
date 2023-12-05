pub struct Config {
    pub ikey: String,
    pub ingestion_endpoint: String,
    pub live_endpoint: String,
}

impl Config {
    pub fn new(connstring: String) -> Config {
        let mut ikey = "";
        let mut ingestion_endpoint = "";
        let mut live_endpoint = "";
        for part in connstring.split(";") {
            println!("{:?}", part);
            let (key, value) = part.split_once("=").unwrap();
            match key {
                "InstrumentationKey" => ikey = value,
                "IngestionEndpoint" => ingestion_endpoint = value,
                "LiveEndpoint" => live_endpoint = value,
                _ => ()
            }
        }

        Config {
            ikey: ikey.to_string(),
            ingestion_endpoint: ingestion_endpoint.to_string(),
            live_endpoint: live_endpoint.to_string(),
        }
    }
}
