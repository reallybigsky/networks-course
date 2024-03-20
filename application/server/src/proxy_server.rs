use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::string::ToString;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use http_body_util::{Empty, Full};
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{HeaderMap, http, Method, Request, Response, StatusCode, Uri};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
struct Blacklist {
    list: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct CachedFileMeta {
    uri: String,
    filename: String,
    last_modified: String,
    etag: String,
}

struct ProxyData {
    log_file: Mutex<tokio::fs::File>,
    cache_meta: Mutex<HashMap<String, CachedFileMeta>>,
    blacklist: Blacklist,
}

#[derive(Clone)]
struct ProxyRequest {
    headers: http::request::Parts,
    body: Bytes,
}

pub struct ProxyServer {
    listener: TcpListener,
    data: Arc<ProxyData>,
}

impl ProxyServer {
    const IP: &'static str = "127.0.0.1";
    const CACHE_DIR: &'static str = "./cache/";
    const CACHE_META: &'static str = "cache_meta.json";
    const BAN_LIST: &'static str = "blacklist.json";

    async fn read_json<T: serde::de::DeserializeOwned + Default>(filename: String) -> io::Result<T> {
        let mut file = tokio::fs::File::open(filename).await?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data).await?;
        serde_json::from_reader(file.into_std().await).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
    
    async fn create_data() -> io::Result<ProxyData> {
        let _ = tokio::fs::create_dir(Self::CACHE_DIR).await;
        let log_filename = "proxy_session_".to_owned() + &SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs().to_string() + ".log";
        let cache_meta: HashMap<String, CachedFileMeta> = Self::read_json(Self::CACHE_META.to_string()).await.unwrap_or_default();
        let blacklist: Blacklist = Self::read_json(Self::BAN_LIST.to_string()).await.unwrap_or_default();

        Ok(ProxyData {
            log_file: Mutex::new(tokio::fs::File::create(log_filename).await?),
            cache_meta: Mutex::new(cache_meta),
            blacklist,
        })
    }

    pub async fn bind(port: &str) -> io::Result<Self> {
        let address = format!("{}:{}", Self::IP, port);
        let listener = TcpListener::bind(&address).await?;
        let data = Arc::new(Self::create_data().await?);
        Ok(Self { listener, data })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let io = TokioIo::new(stream);
            let data = Arc::clone(&self.data);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| {
                        Self::handle_connection(req, data.clone())
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

    async fn log_request(request_headers: http::request::Parts, response_status: StatusCode, proxy_data: Arc<ProxyData>) -> io::Result<()> {
        let curr_time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let log_str = format!("{} {} {} {}\n", curr_time_ms, request_headers.method, response_status, request_headers.uri);
        proxy_data.log_file.lock().await.write_all(log_str.as_ref()).await
    }

    async fn handle_connection(req: Request<Incoming>, proxy_data: Arc<ProxyData>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        match req.method() {
            &Method::GET | &Method::POST => {
                match Self::proxy_client(req, proxy_data).await {
                    Ok(res) => Ok(res),
                    Err(err) if err.kind() == ErrorKind::PermissionDenied => {
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
            &Method::CONNECT => {
                let mut response = Response::new(Self::full(Self::format_error(StatusCode::PROXY_AUTHENTICATION_REQUIRED, "HTTPS is not supported").await));
                *response.status_mut() = StatusCode::PROXY_AUTHENTICATION_REQUIRED;
                Ok(response)
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

    async fn create_request(headers: http::request::Parts, body: Bytes, entry: Option<CachedFileMeta>) -> io::Result<Request<Full<Bytes>>> {
        let mut builder = Request::builder()
            .method(&headers.method)
            .uri(&headers.uri);

        builder.headers_mut().ok_or(Error::new(ErrorKind::InvalidData, ""))?.extend(headers.headers);

        if let Some(entry) = entry {
            builder = builder
                .header("if-modified-since", entry.last_modified)
                .header("if-none-match", entry.etag);
        }

        let request = builder
            .body(Full::new(body))
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;

        Ok(request)
    }

    fn hash_str(str: &str) -> String {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        hasher.write(str.as_bytes());
        hasher.finish().to_string()
    }
    
    async fn put_in_cache(headers: HeaderMap, cache_key: String, proxy_data: Arc<ProxyData>, data: Bytes) -> io::Result<()> {
        if headers.get("last-modified").is_some() && headers.get("etag").is_some() {
            let last_modified = headers.get("last-modified").unwrap().to_str().unwrap_or_default().to_string();
            let etag = headers.get("etag").unwrap().to_str().unwrap_or_default().to_string();
            
            if etag.contains(['W', '%', ';', '\\', '/']) {
                return Err(Error::new(ErrorKind::Other, "Invalid etag type"))
            }
            let filename = Self::hash_str(&cache_key);
            let meta = CachedFileMeta { uri: cache_key.clone(), filename: filename.clone(), last_modified, etag};

            tokio::fs::write(Self::CACHE_DIR.to_string() + &filename, data).await?;

            if let Ok(cache_map_file) = tokio::fs::File::create(Self::CACHE_META).await {
                let mut cache_map = proxy_data.cache_meta.lock().await;
                cache_map.insert(cache_key.clone(), meta);
                serde_json::ser::to_writer_pretty(cache_map_file.into_std().await, cache_map.deref())?;
                println!("SAVED IN CACHE: {}", cache_key);
            }
        }

        Ok(())
    }

    async fn check_cache_hit(uri: Uri, proxy_data: Arc<ProxyData>) -> Option<(CachedFileMeta, Bytes)> {
        let cache_meta = proxy_data.cache_meta.lock().await;
        let cache_meta_entry = cache_meta.get(&uri.to_string())?;
        let file = tokio::fs::File::open(Self::CACHE_DIR.to_string() + &cache_meta_entry.filename).await.ok()?;
        let mut bytes = Vec::new();
        io::BufReader::new(file).read_to_end(&mut bytes).await.ok()?;
        
        Some((cache_meta_entry.clone(), Bytes::from(bytes)))
    }

    async fn send_request(proxy_request: ProxyRequest, proxy_data: Arc<ProxyData>) -> io::Result<(http::response::Parts, Bytes)> {
        let url = &proxy_request.headers.uri;
        let host = url.host().unwrap_or_default();
        let port = url.port_u16().unwrap_or(80);
        let address = format!("{}:{}", host, port);
        let cache_key = url.to_string();
        
        if Self::check_blacklisted(cache_key.clone(), proxy_data.clone()) {
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

        let (cached_entry, cached_body) = Self::check_cache_hit(url.clone(), proxy_data.clone()).await.unzip();
        let (proxy_request_headers, proxy_request_body) = (proxy_request.headers, proxy_request.body);
        let request = Self::create_request(proxy_request_headers.clone(), proxy_request_body, cached_entry).await?;

        let response = sender.send_request(request).await
            .map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?;

        let (headers, body) = response.into_parts();
        let body = body.collect().await.map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?.to_bytes();

        let _ = Self::log_request(proxy_request_headers.clone(), headers.status, proxy_data.clone()).await;

        match headers.status {
            StatusCode::NOT_MODIFIED if cached_body.is_some() => {
                println!("CACHE HIT: {}", cache_key);
                let mut result = Response::new(cached_body.unwrap());
                *result.status_mut() = StatusCode::OK;
                Ok(result.into_parts())
            }
            _ => {
                let _ = Self::put_in_cache(headers.headers.clone(), cache_key, proxy_data, body.clone()).await;
                Ok((headers, body))
            }
        }
    }

    async fn proxy_client(client_req: Request<Incoming>, proxy_data: Arc<ProxyData>) -> io::Result<Response<BoxBody<Bytes, hyper::Error>>> {
        let (headers, body) = client_req.into_parts();
        let body = body.collect().await.map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?.to_bytes();

        let proxy_request = ProxyRequest { headers, body };
        let (response_headers, response_body) = Self::send_request(proxy_request.clone(), proxy_data.clone()).await?;

        let mut result = Response::builder().status(response_headers.status).version(response_headers.version);
        result.headers_mut().ok_or(Error::new(ErrorKind::InvalidData, ""))?.extend(response_headers.headers);
        result.extensions_mut().ok_or(Error::new(ErrorKind::InvalidData, ""))?.extend(response_headers.extensions);
        result.body(Self::full(response_body)).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}
