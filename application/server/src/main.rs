use std::env;
use crate::proxy_server::ProxyServer;

mod proxy_server;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(n_workers)
        .enable_all()
        .build()
        .expect("Cannot start server runtime");
    
    rt.block_on(async {
        let server = ProxyServer::bind(port).await?;
        server.start().await?;
        Ok(())
    })
}
