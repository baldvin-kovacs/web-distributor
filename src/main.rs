use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, create_dir_all, read_dir, rename};
use std::io::ErrorKind;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    home: String,
    acme_redirect_configs: String,
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
                    home: "/etc/web-distributor".to_string(),
                    acme_redirect_configs: "/etc/acme-redirect.d".to_string(),
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
  
          server_name {from};
  
          return 301 https://$server_name$request_uri;
}}
server {{
        listen 443 ssl http2;
        listen [::]:443 ssl http2;

        server_name {from};

        ssl_certificate /var/lib/acme-redirect/live/{from}/fullchain;
        ssl_certificate_key /var/lib/acme-redirect/live/{from}/privkey;
        ssl_session_timeout 1d;
        ssl_session_cache shared:MozSSL:10m;  # about 40000 sessions
        ssl_session_tickets off;
        
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;

        add_header Strict-Transport-Security \"max-age=63072000\" always;

        ssl_stapling on;
        ssl_stapling_verify on;
        ssl_trusted_certificate /var/lib/acme-redirect/live/{from}/chain;
        resolver 127.0.0.1;


        location / {{
                proxy_pass {to};
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header X-Forwarded-Proto $scheme;
        }}
}}")
}

fn acme_redirect_config_build(namespace: &str) -> String {
    format!(
        "[cert]
name = \"{namespace}\"
dns_names = [
    \"{namespace}\",
]
exec = [
    \"systemctl reload nginx\",
]
"
    )
}

fn generate_webserver_configs(config: &Config) {
    let nginx_folder = Path::new(&config.home).join("nginx");
    let old = Path::new(&config.home).join("nginx-old");

    if let Err(e) = fs::remove_dir_all(&old) {
        if e.kind().ne(&ErrorKind::NotFound) {
            panic!("couldn't delete nginx-old folder");
        }
    }
    if let Err(e) = rename(&nginx_folder, &old) {
        if e.kind().ne(&ErrorKind::NotFound) {
            panic!("couldn't move nginx folder to nginx-old {}", e);
        }
    }

    create_dir_all(&nginx_folder).unwrap();

    for (source, target) in &config.map {
        fs::write(
            nginx_folder.join(format!("{}.nginx", source)),
            nginx_proxy_build(&source, &target),
        )
        .unwrap();
    }
}

fn generate_acme_redirect_config(config: &Config) {
    let acme_path: &Path = Path::new(&config.acme_redirect_configs);
    let old = acme_path.join("old");

    if let Err(e) = fs::remove_dir_all(&old) {
        if e.kind().ne(&ErrorKind::NotFound) {
            panic!("couldn't delete acme-redirect.d/old");
        }
    }
    create_dir_all(&old).unwrap();

    let dir_entries = read_dir(acme_path).unwrap();
    for f in dir_entries {
        let entry = f.unwrap();
        if entry.file_name() == "old"
            || !entry
                .file_name()
                .to_str()
                .unwrap()
                .starts_with("web-distributor")
        {
            continue;
        }
        rename(
            acme_path.join(entry.file_name()),
            old.join(entry.file_name()),
        )
        .unwrap();
    }

    for (namespace, _) in &config.map {
        fs::write(
            acme_path.join(format!("web-distributor.{namespace}.conf")),
            acme_redirect_config_build(&namespace),
        )
        .unwrap();
    }
}

fn main() {
    let binding = std::env::args().nth(1);
    let config_path: &Path = match &binding {
        Some(arg) => Path::new(arg),
        None => Path::new("/etc/web-distributor.toml"),
    };

    let config = read_config(&config_path);

    generate_webserver_configs(&config);

    generate_acme_redirect_config(&config);
}
