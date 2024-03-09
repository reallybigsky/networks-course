# How to run

`cargo`(rust packet manager) is required to run this application.

Get file request must look like:
```http request
http://<server_ip>:<server_port>/files?path=<filepath>
```


## Server

```shell
cd server
cargo run -q --package server --bin server -- <server_port> <concurrency_level>
```
## Client
```shell
cd client
cargo run -q --package client --bin client -- <server_addr> <server_port> <filename>
```
The contents of the file will be printed to the console if there were no errors. Thus, it is necessary to request only text files.