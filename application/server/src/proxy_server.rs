use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufReader, Error, ErrorKind, Read, Write};
use std::ops::Deref;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use http_body_util::{Empty, Full};
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{HeaderMap, http, Method, Request, Response, StatusCode};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use url::{Position, Url};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
struct Blacklist {
    list: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct CachedFileMeta {
    last_modified: String,
    etag: String,
}

struct ProxyData {
    log_file: Mutex<std::fs::File>,
    cache_meta_path: Mutex<String>,
    cache_meta: Mutex<HashMap<String, CachedFileMeta>>,
    blacklist: Blacklist,
}

#[derive(Clone)]
struct ProxyRequest {
    headers: http::request::Parts,
    body: Bytes,
}

pub struct ProxyServer {
    address: Arc<String>,
    listener: TcpListener,
    data: Arc<ProxyData>,
}

impl ProxyServer {
    const IP: &'static str = "127.0.0.1";
    const CACHE_DIR: &'static str = "./cache/";
    const CACHE_META: &'static str = "cache_meta.json";
    const BAN_LIST: &'static str = "blacklist.json";

    async fn create_data() -> io::Result<ProxyData> {
        let log_filename = "proxy_session_".to_owned() + &SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string() + ".log";
        let _ = std::fs::create_dir(Self::CACHE_DIR);
        let cache_file = OpenOptions::new()
            .read(true)
            .open(Self::CACHE_META)?;

        let cache_meta: HashMap<String, CachedFileMeta> = serde_json::from_reader(cache_file).unwrap_or_default();

        let blacklist: Blacklist = if let Ok(banned_list_file) = std::fs::File::open(Self::BAN_LIST) {
            serde_json::from_reader::<std::fs::File, Blacklist>(banned_list_file).unwrap_or_default()
        } else {
            Blacklist::default()
        };

        Ok(ProxyData {
            log_file: Mutex::new(std::fs::File::create(log_filename)?),
            cache_meta_path: Mutex::new(Self::CACHE_META.to_string()),
            cache_meta: Mutex::new(cache_meta),
            blacklist,
        })
    }

    pub async fn bind(port: &str) -> io::Result<Self> {
        let address = format!("{}:{}", Self::IP, port);
        let listener = TcpListener::bind(&address).await?;
        let data = Arc::new(Self::create_data().await?);
        Ok(Self { address: Arc::new(address), listener, data })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let io = TokioIo::new(stream);
            let address = Arc::clone(&self.address);
            let data = Arc::clone(&self.data);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| {
                        Self::handle_connection(req, address.clone(), data.clone())
                    }))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    async fn format_error(status: StatusCode, message: &str) -> String {
        format!("<!DOCTYPE html>
    <html lang=\"en\">
      <head>
        <meta charset=\"utf-8\">
        <title>{status}</title>
      </head>
      <body>
        <h1>Oops, {status}</h1>
        <p>{message}</p>
      </body>
    </html>")
    }

    async fn log_request(method: Method, url: String, status_code: StatusCode, proxy_data: Arc<ProxyData>) -> io::Result<()> {
        let curr_time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let log_str = format!("{} {} {} {}\n", curr_time_ms, method.as_str(), status_code.as_str(), url);
        proxy_data.log_file.lock().await.write_all(log_str.as_ref())
    }

    async fn handle_connection(req: Request<Incoming>, server_addr: Arc<String>, proxy_data: Arc<ProxyData>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let addr = "http://".to_owned() + &server_addr.to_string() + "/";
        match (req.method(), req.uri().path_and_query()) {
            (method, Some(url)) if method == Method::GET || method == Method::POST => {
                // Раст и его либы не могут в работу с путями в http запросах (как и я мб)
                // Сделал так, как чувствую
                let mut arg = url.as_str()[1..].to_string();
                let referer = match req.headers().get("referer") {
                    Some(referer) if referer.to_str().is_ok() => {
                        let referer = referer.to_str().unwrap();
                        let it = referer.strip_prefix(&addr).unwrap_or(referer);
                        if let Some(stripped) = url.as_str()[1..].strip_prefix(it) {
                            arg = "/".to_string() + stripped
                        } else if !arg.starts_with("http://") && !arg.starts_with("https://") {
                            arg = "/".to_string() + &arg
                        }
                        it.to_string()
                    }
                    _ => String::new()
                };
                match Self::proxy_client(req, arg, referer, proxy_data).await {
                    Ok(res) => Ok(res),
                    Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                        println!("Err: {:?}", err);
                        let mut response = Response::new(Self::full(Self::format_error(StatusCode::FORBIDDEN, "This site is blacklisted").await));
                        *response.status_mut() = StatusCode::FORBIDDEN;
                        Ok(response)
                    }
                    Err(err) => {
                        println!("Err: {:?}", err);
                        let mut response = Response::new(Self::full(Self::format_error(StatusCode::NOT_FOUND, &err.to_string()).await));
                        *response.status_mut() = StatusCode::NOT_FOUND;
                        Ok(response)
                    }
                }
            }
            _ => {
                let mut not_found = Response::new(Self::empty());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }

    fn empty() -> BoxBody<Bytes, hyper::Error> {
        Empty::<Bytes>::new()
            .map_err(|never| match never {})
            .boxed()
    }

    fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
        Full::new(chunk.into())
            .map_err(|never| match never {})
            .boxed()
    }

    fn check_blacklisted(arg: String, proxy_data: Arc<ProxyData>) -> bool {
        let mut curr_url = String::new();
        for segment in arg.split_inclusive('/') {
            curr_url += segment;
            if proxy_data.blacklist.list.contains(&curr_url) {
                return true;
            }
        }

        false
    }

    async fn create_request(proxy_request: ProxyRequest, host: String, arg: String, proxy_data: Arc<ProxyData>, cache_key: Option<String>) -> io::Result<Request<Full<Bytes>>> {
        let mut builder = Request::builder()
            .method(&proxy_request.headers.method)
            .uri(arg)
            .header(hyper::header::HOST, host);

        for (key, value) in proxy_request.headers.headers.iter() {
            if key.as_str() == "host" {
                continue;
            }
            builder = builder.header(key, value);
        }

        if let Some(cache_key_value) = cache_key {
            let cache_entry = if proxy_request.headers.method == Method::GET {
                proxy_data.cache_meta.lock().await.get(&cache_key_value).cloned()
            } else {
                None
            };

            if cache_entry.is_some() {
                builder = builder
                    .header("if-modified-since", cache_entry.clone().unwrap().last_modified)
                    .header("if-none-match", cache_entry.clone().unwrap().etag);
            }
        }

        let request = builder
            .body(Full::new(proxy_request.body))
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
        
        Ok(request)
    }

    async fn put_in_cache(headers: HeaderMap, cache_key: String, proxy_data: Arc<ProxyData>, data: Bytes) -> io::Result<()> {
        if headers.get("last-modified").is_some() && headers.get("etag").is_some() {
            let last_modified = headers.get("last-modified").unwrap().to_str().unwrap_or_default().to_string();
            let etag = headers.get("etag").unwrap().to_str().unwrap_or_default().to_string();
            let meta = CachedFileMeta { last_modified, etag: etag.clone() };

            if let Ok(cache_map_json) = std::fs::File::create(proxy_data.cache_meta_path.lock().await.as_str()) {
                let mut data_file = std::fs::File::create(Self::CACHE_DIR.to_string() + &etag.replace('\"', ""))?;
                data_file.write_all(&data)?;

                let mut cache_map = proxy_data.cache_meta.lock().await;
                cache_map.insert(cache_key.clone(), meta);
                serde_json::ser::to_writer_pretty(cache_map_json, cache_map.deref())?;
                println!("SAVED IN CACHE: {}", cache_key);
            }
        }

        Ok(())
    }

    async fn check_cache_hit(cache_key: String, proxy_data: Arc<ProxyData>) -> Option<BoxBody<Bytes, hyper::Error>> {
        let cache_meta = proxy_data.cache_meta.lock().await;
        let Some(cache_map_json) = cache_meta.get(&cache_key) else {
            return None;
        };
        let Ok(file) = std::fs::File::open(Self::CACHE_DIR.to_string() + &cache_map_json.etag.replace('\"', "")) else {
            return None;
        };
        let mut bytes = Vec::new();
        let Ok(_) = BufReader::new(file).read_to_end(&mut bytes) else {
            return None;
        };

        Some(Self::full(bytes))
    }

    async fn send_request(proxy_request: ProxyRequest, mut arg: String, referer: String, proxy_data: Arc<ProxyData>) -> io::Result<(http::response::Parts, BoxBody<Bytes, hyper::Error>)> {
        let url = if !referer.is_empty() {
            Url::parse(&referer).map_err(|err| Error::new(ErrorKind::InvalidInput, err))?
        } else {
            let url = Url::parse(&arg).map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
            arg = url[Position::BeforePath..].to_string();
            url
        };

        let cache_key = url.to_string() + " +++ " + &arg;
        let Some(host) = url.host() else {
            return Err(Error::new(ErrorKind::InvalidInput, ""));
        };
        let port = url.port().unwrap_or(80);
        let address = format!("{}:{}", host, port);

        if Self::check_blacklisted(url[..Position::BeforeQuery].to_string(), proxy_data.clone()) {
            return Err(Error::new(ErrorKind::PermissionDenied, ""));
        }

        let io = TokioIo::new(TcpStream::connect(address).await?);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await
            .map_err(|err| Error::new(ErrorKind::ConnectionAborted, err))?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let request_method = proxy_request.headers.method.clone();
        let cached_body = Self::check_cache_hit(cache_key.clone(), proxy_data.clone()).await;

        let request = if cached_body.is_some() {
            Self::create_request(proxy_request, host.to_string(), arg, proxy_data.clone(), Some(cache_key.clone())).await?
        } else {
            Self::create_request(proxy_request, host.to_string(), arg, proxy_data.clone(), None).await?
        };

        let response = sender.send_request(request).await
            .map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?;

        let (headers, body) = response.into_parts();
        let body = body.collect().await.map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?.to_bytes();

        let _ = Self::log_request(request_method, cache_key.clone(), headers.status, proxy_data.clone()).await;

        match headers.status {
            StatusCode::NOT_MODIFIED if cached_body.is_some() => {
                println!("CACHE HIT: {}", cache_key);
                let mut result = Response::new(cached_body.unwrap());
                *result.status_mut() = StatusCode::OK; 
                Ok(result.into_parts())
            }
            _ => {
                let _ = Self::put_in_cache(headers.headers.clone(), cache_key, proxy_data, body.clone()).await;
                Ok((headers, Self::full(body)))
            }
        }
    }

    async fn handle_redirect(proxy_request: ProxyRequest, mut response_parts: http::response::Parts, proxy_data: Arc<ProxyData>) -> io::Result<(http::response::Parts, BoxBody<Bytes, hyper::Error>)> {
        const REDIRECTION_LIMIT: i32 = 10;

        let mut response_body = Self::empty();
        // finite loop to prevent endless redirections and ddos ban
        for _ in 0..REDIRECTION_LIMIT {
            match response_parts.headers.get("location") {
                Some(location) if location.to_str().is_ok() => {
                    let location_str = location.to_str().map_or("".to_string(), |s| s.to_string());
                    (response_parts, response_body) = Self::send_request(proxy_request.clone(), location_str, String::new(), proxy_data.clone()).await?;
                    if response_parts.status != StatusCode::MOVED_PERMANENTLY && response_parts.status != StatusCode::FOUND {
                        return Ok((response_parts, response_body));
                    }
                }
                _ => { return Ok((response_parts, response_body)); }
            };
        }
        Ok((response_parts, response_body))
    }

    async fn proxy_client(client_req: Request<Incoming>, client_arg: String, client_referer: String, proxy_data: Arc<ProxyData>) -> io::Result<Response<BoxBody<Bytes, hyper::Error>>> {
        let (headers, body) = client_req.into_parts();
        let body = body.collect().await.map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?.to_bytes();
        
        let proxy_request = ProxyRequest { headers, body };
        let (mut response_headers, mut response_body) = Self::send_request(proxy_request.clone(), client_arg, client_referer, proxy_data.clone()).await?;

        if response_headers.status == StatusCode::MOVED_PERMANENTLY || response_headers.status == StatusCode::FOUND {
            (response_headers, response_body) = Self::handle_redirect(proxy_request, response_headers, proxy_data).await?;
        }

        let mut result = Response::builder()
            .status(response_headers.status)
            .version(response_headers.version);
        result.headers_mut().ok_or(Error::new(ErrorKind::InvalidData, ""))?.extend(response_headers.headers);
        result.extensions_mut().ok_or(Error::new(ErrorKind::InvalidData, ""))?.extend(response_headers.extensions);
        result.body(response_body).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}
