use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    map: HashMap<String, String>
}

fn main() {
    let homepath: &Path = Path::new(".");
    let config_path = homepath.join("config");

    let config: Config = match fs::read_to_string(&config_path) {
      Ok(config_string) => {
        let c: Config = toml::from_str(&config_string).unwrap();
        c
      }
      Err(e) => {
        match e.kind() {
          ErrorKind::NotFound => {
            Config {map: HashMap::new()}
          }
          _=> {
            panic!();
          }
        }
      }
    };

    for (key, value) in &config.map {
        println!("{}, {}", key, value);
    }

    let _ = fs::write(&config_path, toml::to_string(&config).unwrap());
}
