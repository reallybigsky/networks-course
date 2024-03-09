use std::env;
use std::io::ErrorKind;
use crate::http_client::HttpClient;

mod http_client;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        panic!("Must specify 4 arguments")
    }

    let server_addr = args.get(1)
        .expect("Must specify 4 arguments");
    let server_port = args.get(2)
        .expect("Must specify 4 arguments");
    let filename = args.get(3)
        .expect("Must specify 4 arguments");

    let mut client = HttpClient::bind(server_addr, server_port)
        .expect("Cannot bind client socket");

    match client.request_file(filename) {
        Ok(content) => {
            // Просто выводим на экран
            let str_content = String::from_utf8(content).expect("Content is not a valid UTF8 string");
            println!("{str_content}");
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            println!("File not found: {err:?}")
        }
        Err(err) => {
            println!("Cannot get file: {err:?}")
        }
    }
}
