use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    map: HashMap<String, String>,
}

fn nginx_proxy_build(from: &str, to: &str) -> String {
    format!(
        "server {{
      listen 80;
      listen [::]:80;

      server_name {};

      location / {{
              proxy_pass http://{}/$request_uri;
      }}
}}",
        from, to
    )
}

fn main() {
    let homepath: &Path = Path::new(".");
    let config_path = homepath.join("config.toml");
    let nginx_folder = homepath.join("nginx");

    let config: Config = match fs::read_to_string(&config_path) {
        Ok(config_string) => {
            let c: Config = toml::from_str(&config_string).unwrap();
            c
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Config {
                map: HashMap::new(),
            },
            _ => {
                panic!();
            }
        },
    };

    for (key, value) in &config.map {
        println!("{}, {}", key, value);
        let _ = fs::write(nginx_folder.join(key), nginx_proxy_build(key, value));
    }

    let _ = fs::write(&config_path, toml::to_string(&config).unwrap());
}
