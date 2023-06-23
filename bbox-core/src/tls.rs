use crate::app_dir;
use crate::config::{config_error_exit, error_exit};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{fs::File, io::BufReader};

// For self-signed certificates we recommend to use [`mkcert`].
// To use local CA, you should run:
//
// ```sh
// mkcert -install
// ```
//
// If you want to generate your own cert/private key file, then run:
//
// ```sh
// mkcert localhost 127.0.0.1
// ```
//
// [`mkcert`]: https://github.com/FiloSottile/mkcert

pub fn load_rustls_config(tls_cert: &str, tls_key: &str) -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(app_dir(tls_cert)).unwrap_or_else(error_exit));
    let key_file = &mut BufReader::new(File::open(app_dir(tls_key)).unwrap_or_else(error_exit));

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap_or_else(error_exit)
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap_or_else(error_exit)
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        config_error_exit("Could not locate PKCS 8 private keys.");
    }

    config
        .with_single_cert(cert_chain, keys.remove(0))
        .unwrap_or_else(error_exit)
}
