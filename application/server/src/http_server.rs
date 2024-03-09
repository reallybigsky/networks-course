use std::fs;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use threadpool::ThreadPool;
use url::Url;

pub struct HttpServer {
    listener: TcpListener,
    th_pool: ThreadPool,
}

impl HttpServer {
    const MAX_FILE_SIZE_BYTES: u64 = 1u64 << 20;
    const IP: &'static str = "127.0.0.1";

    pub fn bind(port: &str, n_workers: usize) -> std::io::Result<Self> {
        let address = format!("{}:{}", Self::IP, port);
        let listener = TcpListener::bind(address)?;
        let th_pool = ThreadPool::new(n_workers);
        Ok(Self { listener, th_pool })
    }

    pub fn start(&self) {
        loop {
            let (connection, _) = match self.listener.accept() {
                Ok((conn, addr)) => {
                    println!("new client: {addr:?}");
                    (conn, addr)
                }
                Err(e) => {
                    println!("couldn't get client: {e:?}");
                    continue;
                }
            };

            self.th_pool.execute(|| Self::handle_connection(connection))
        }
    }

    fn process_request(request: &[String]) -> std::io::Result<PathBuf> {
        let lines: Vec<_> = request.iter().map(|line| line.split_whitespace().collect::<Vec<_>>()).collect();

        for line in lines {
            if line.len() == 3 &&
                line.first().is_some_and(|s| *s == "GET") &&
                line.get(2).is_some_and(|s| s.starts_with("HTTP/1."))
            {
                let path = line.get(1).unwrap();
                if !path.starts_with("/files?") {
                    return Err(Error::new(ErrorKind::NotFound, "Invalid request path"));
                }

                let Ok(url) = Url::try_from(format!("http://0.0.0.0{path}").as_str()) else {
                    return Err(Error::new(ErrorKind::NotFound, "Invalid request path"));
                };

                for (key, value) in url.query_pairs() {
                    if key == "path" {
                        let meta = fs::metadata(value.as_ref()).map_err(|_| Error::new(ErrorKind::NotFound, "Invalid filepath"))?;
                        if !meta.is_file() {
                            return Err(Error::new(ErrorKind::NotFound, "Not a file"));
                        }

                        if meta.len() > Self::MAX_FILE_SIZE_BYTES {
                            return Err(Error::new(ErrorKind::NotFound, "File too big"));
                        }

                        return Ok(PathBuf::from(value.as_ref()));
                    }
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound, "Invalid request"))
    }

    fn send_not_found_error(mut stream: &TcpStream, message: &str) -> std::io::Result<()> {
        let status = "HTTP/1.1 404 NOT FOUND";

        // Добавим описание ошибки в виде html файла
        let content = format!("<!DOCTYPE html>
    <html lang=\"en\">
      <head>
        <meta charset=\"utf-8\">
        <title>NOT FOUND</title>
      </head>
      <body>
        <h1>Oops, 404 NOT FOUND!</h1>
        <p>{message}</p>
      </body>
    </html>");

        let response = format!("{status}\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
        stream.write_all(response.as_bytes())
    }

    fn send_file(mut stream: &TcpStream, path: PathBuf) -> std::io::Result<()> {
        let status = "HTTP/1.1 200 OK";
        let content = fs::read(path)?;
        let response_header = format!("{status}\r\nContent-Length: {}\r\n\r\n", content.len());
        stream.write_all(response_header.as_bytes())?;
        stream.write_all(&content)?;
        stream.flush()
    }

    pub fn handle_connection(mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(Result::unwrap_or_default)
            .take_while(|line| !line.is_empty())
            .collect();

        match Self::process_request(&http_request) {
            Ok(path) => {
                Self::send_file(&stream, path).unwrap()
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                Self::send_not_found_error(&stream, &err.to_string()).unwrap_or_default()
            }
            Err(err) => {
                println!("Unexpected error: {err:?}")
            }
        }
    }
}
