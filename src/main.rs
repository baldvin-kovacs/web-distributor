use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{format, write};
use std::fs::{self, create_dir, rename};
use std::io::ErrorKind;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    map: HashMap<String, String>,
}

fn read_config(config_path: &Path) -> Config {
    match fs::read_to_string(&config_path) {
        Ok(config_string) => {
            let c: Config = toml::from_str(&config_string).unwrap();
            c
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                let config = Config {
                    map: HashMap::new(),
                };
                let _ = fs::write(&config_path, toml::to_string(&config).unwrap());
                config
            }
            _ => {
                panic!();
            }
        },
    }
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

// fn request_cert(hostname: &str) {}

// fn certificates(config: &Config) {}

fn generate_configs(homepath: &Path, config: &Config) {
    let nginx_folder = homepath.join("nginx");
    let old = homepath.join("nginx-old");

    if let Err(e) = rename(&nginx_folder, &old) {
        if e.kind().ne(&ErrorKind::NotFound) {
            panic!("couldn't move nginx folder to nginx-old {}", e);
        }
    }

    create_dir(&nginx_folder).unwrap();

    for (source, target) in &config.map {
        fs::write(nginx_folder.join(format!("{}.nginx", source)), nginx_proxy_build(&source, &target)).unwrap();
    }
}

fn main() {
    let homepath: &Path = Path::new(".");
    let config_path = homepath.join("config.toml");

    let config = read_config(&config_path);

    generate_configs(&homepath,&config);
    // certificates(&config);
    
}
