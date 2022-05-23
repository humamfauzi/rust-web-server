use std::net::TcpListener;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs;
use hello::ThreadPool;
use hello::{Route, Routes, RoutesProperties};
fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        })
    }
}


fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    // if when reading stream there is an error; assume it is an empty bytes
    if let Err(_e) = stream.read(&mut buffer) {
        buffer = [0;1024];
    };

    // routes should be generated once, and reusable yet immutable throughout operation
    let routes = get_routes();
    let matched_routes = routes.find_route_by_buffer(&buffer);
    let route = &matched_routes[0];
    let (status_line, filename) = route.reply();
    let contents = match fs::read_to_string(filename).ok() {
        None => "".to_string(),
        Some(content) => content,
    };
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_routes() -> Routes<'static> {
    let routes = vec![
        Route::new("HTTP/1.1", "/", "GET", "HTTP/1.1 200 OK", "hello.html"),
        Route::new("HTTP/1.1", "/", "POST", "HTTP/1.1 200 OK", "hello.html"),
        Route::new("HTTP/1.1", "/sleep", "GET", "HTTP/1.1 200 OK", "404.html"),
    ];
    routes
}