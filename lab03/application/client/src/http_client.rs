use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct HttpClient {
    stream: TcpStream,
}

impl HttpClient {
    pub fn bind(server_addr: &str, server_port: &str) -> std::io::Result<Self> {
        Ok(Self { stream: TcpStream::connect(format!("{}:{}", server_addr, server_port))? })
    }

    pub fn request_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
        let request = format!("GET /files?path={filename} HTTP/1.1\r\n\r\n");
        self.stream.write_all(request.as_bytes())?;
        self.stream.flush()?;
        let mut buf_reader = BufReader::new(&mut self.stream);
        
        let mut status = "200".to_string();
        let mut content_len = None;
        loop {
            let mut line = String::new();
            if buf_reader.read_line(&mut line).unwrap_or(0) == 0 || line.split_whitespace().next().is_none() {
                break
            }
            
            if line.starts_with("HTTP/1.1") {
                status = line.split_whitespace().skip(1).take(1).next().unwrap_or("404").to_string();
            } else if line.starts_with("Content-Length:") {
                content_len = line.split_whitespace().last().and_then(|s| s.parse::<usize>().ok())
            }
        }
        
        if status != "200" || content_len.is_none() {
            return Err(Error::new(ErrorKind::NotFound, "Cannot get file"));
        }

        let content_len = content_len.unwrap_or_default();
        let mut content: Vec<u8> = Vec::new();
        content.resize(content_len, 0);
        buf_reader.read_exact(&mut content)?;
        Ok(content)
    }
}