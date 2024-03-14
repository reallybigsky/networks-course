use std::env;
use crate::http_server::HttpServer;

mod http_server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("Must specify 2 arguments")
    }
    
    let port = args.get(1)
        .expect("Must specify 2 arguments");
    let n_workers = args.get(2)
        .expect("Must specify 2 arguments")
        .parse::<usize>()
        .expect("concurrency level must be a number");
    let server = HttpServer::bind(port, n_workers)
        .expect("Cannot bind server");
    
    server.start();
}
