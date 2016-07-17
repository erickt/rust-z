// native-tls configuration copied from rustup

pub mod hyper {

    extern crate hyper;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    extern crate openssl_sys;
    extern crate native_tls;

    use std::io;
    use std::time::Duration;
    use url::Url;
    use errors::*;
    use super::hyper_base;
    use self::hyper::error::Result as HyperResult;
    use self::hyper::net::{SslClient, NetworkStream};
    use self::hyper::client::Client;
    use std::io::Result as IoResult;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, Shutdown};
    use std::sync::{Arc, Mutex, MutexGuard};

    pub fn download(url: &Url) -> Result<Client> {
        hyper_base::download::<NativeSslClient>(url)
    }

    struct NativeSslClient;

    impl hyper_base::NewSslClient for NativeSslClient {
        fn new() -> Self { NativeSslClient }
        fn maybe_init_certs() { maybe_init_certs() }
    }

    impl<T: NetworkStream + Send + Clone> SslClient<T> for NativeSslClient {
        type Stream = NativeSslStream<T>;

        fn wrap_client(&self, stream: T, host: &str) -> HyperResult<Self::Stream> {
            use self::native_tls::ClientBuilder as TlsClientBuilder;
            use self::hyper::error::Error as HyperError;

            let mut ssl_builder = try!(TlsClientBuilder::new()
                                       .map_err(|e| HyperError::Ssl(Box::new(e))));
            let ssl_stream = try!(ssl_builder.handshake(host, stream)
                                  .map_err(|e| HyperError::Ssl(Box::new(e))));

            Ok(NativeSslStream(Arc::new(Mutex::new(ssl_stream))))
        }
    }

    #[derive(Clone)]
    struct NativeSslStream<T>(Arc<Mutex<native_tls::TlsStream<T>>>);

    #[derive(Debug)]
    struct NativeSslPoisonError;

    impl ::std::error::Error for NativeSslPoisonError {
        fn description(&self) -> &str { "mutex poisoned during TLS operation" }
    }

    impl ::std::fmt::Display for NativeSslPoisonError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
            f.write_str(::std::error::Error::description(self))
        }
    }

    impl<T> NativeSslStream<T> {
        fn lock<'a>(&'a self) -> IoResult<MutexGuard<'a, native_tls::TlsStream<T>>> {
            self.0.lock()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, NativeSslPoisonError))
        }
    }

    impl<T> NetworkStream for NativeSslStream<T>
        where T: NetworkStream
    {
        fn peer_addr(&mut self) -> IoResult<SocketAddr> {
            self.lock()
                .and_then(|mut t| t.get_mut().peer_addr())
        }
        fn set_read_timeout(&self, dur: Option<Duration>) -> IoResult<()> {
            self.lock()
                .and_then(|t| t.get_ref().set_read_timeout(dur))
        }
        fn set_write_timeout(&self, dur: Option<Duration>) -> IoResult<()> {
            self.lock()
                .and_then(|t| t.get_ref().set_write_timeout(dur))
        }
        fn close(&mut self, how: Shutdown) -> IoResult<()> {
            self.lock()
                .and_then(|mut t| t.get_mut().close(how))
        }
    }

    impl<T> Read for NativeSslStream<T>
        where T: Read + Write
    {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            self.lock()
                .and_then(|mut t| t.read(buf))
        }
    }

    impl<T> Write for NativeSslStream<T>
        where T: Read + Write
    {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.lock()
                .and_then(|mut t| t.write(buf))
        }
        fn flush(&mut self) -> IoResult<()> {
            self.lock()
                .and_then(|mut t| t.flush())
        }
    }

    // Tell our statically-linked OpenSSL where to find root certs
    // cc https://github.com/alexcrichton/git2-rs/blob/master/libgit2-sys/lib.rs#L1267
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn maybe_init_certs() {
        use std::sync::{Once, ONCE_INIT};
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| {
            openssl_sys::probe::init_ssl_cert_env_vars();
        });
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    fn maybe_init_certs() { }
}

pub mod hyper_base {

    extern crate hyper;

    use std::io;
    use url::Url;
    use errors::*;
    use self::hyper::net::{SslClient, HttpStream};
    use self::hyper::client::Client;

    pub trait NewSslClient {
        fn new() -> Self;
        fn maybe_init_certs();
    }

    pub fn download<S>(url: &Url) -> Result<Client>
        where S: SslClient<HttpStream> + NewSslClient + Send + Sync + 'static,
    {

        use self::hyper::client::{Client, ProxyConfig};
        use self::hyper::header::ContentLength;
        use self::hyper::net::{HttpsConnector};

        // The Hyper HTTP client
        let client;

        S::maybe_init_certs();

        if url.scheme() == "https" {
            // Connect with hyper + native_tls
            client = Client::with_connector(HttpsConnector::new(S::new()));
        } else if url.scheme() == "http" {
            client = Client::new();
        } else {
            return Err(format!("unsupported URL scheme: '{}'", url.scheme()).into());
        }

        Ok(client)
    }

    fn proxy_from_env(url: &Url) -> Option<(String, u16)> {
        use std::env::var_os;

        let mut maybe_https_proxy = var_os("https_proxy").map(|ref v| v.to_str().unwrap_or("").to_string());
        if maybe_https_proxy.is_none() {
            maybe_https_proxy = var_os("HTTPS_PROXY").map(|ref v| v.to_str().unwrap_or("").to_string());
        }
        let maybe_http_proxy = var_os("http_proxy").map(|ref v| v.to_str().unwrap_or("").to_string());
        let mut maybe_all_proxy = var_os("all_proxy").map(|ref v| v.to_str().unwrap_or("").to_string());
        if maybe_all_proxy.is_none() {
            maybe_all_proxy = var_os("ALL_PROXY").map(|ref v| v.to_str().unwrap_or("").to_string());
        }
        if let Some(url_value) = match url.scheme() {
            "https" => maybe_https_proxy.or(maybe_http_proxy.or(maybe_all_proxy)),
            "http" => maybe_http_proxy.or(maybe_all_proxy),
            _ => maybe_all_proxy,
        } {
            if let Ok(proxy_url) = Url::parse(&url_value) {
                if let Some(host) = proxy_url.host_str() {
                    let port = proxy_url.port().unwrap_or(8080);
                    return Some((host.to_string(), port));
                }
            }
        }
        None
    }
}
