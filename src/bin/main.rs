use std::net::TcpListener;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::fs;
use hello::ThreadPool;

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
#[derive(Debug, Copy, Clone)]
struct Route<'a> {
    version: &'a str,
    path: &'a str,
    method: &'a str,
    reply: &'a str,
    filename: &'a str,
}

impl<'a> Route<'a> {
    fn new(version: &'a str, path: &'a str, method: &'a str, reply: &'a str, filename: &'a str) -> Route<'a> {
        Route{ 
            version,
            path,
            method,
            reply,
            filename,
        }
    }
    fn to_complete_string(&self) -> String {
        let check = format!("{} {} {}", self.method, self.path, self.version);
        check
    }

    fn reply(&self) ->  (&str,  &str) {
        (self.reply, self.filename)
    }
}

type Routes<'a> = Vec<Route<'a>>;
trait RoutesProperties {
    fn find_route_by_version(&mut self, version: &str) -> Routes;
    fn find_route_by_method(&self, method: &str) -> Routes;
    fn find_route_by_path(&self, path: &str) -> Routes;
    fn find_route_by_buffer(&self, buff: &[u8]) -> Routes;
}

impl<'a> RoutesProperties for Routes<'a> {
    fn find_route_by_version(&mut self, version: &str) -> Routes {
        let mut vec_routes: Routes = vec![];
        for route in self {
            if route.version == version {
                vec_routes.push(route.clone())
            }
        }
        vec_routes
    }
    
    fn find_route_by_method(&self, method: &str) -> Routes {
        let mut vec_routes: Routes = vec![];
        for route in self {
            if route.method == method {
                vec_routes.push(route.clone())
            }
        }
        vec_routes
   }
    
    fn find_route_by_path(&self, path: &str) -> Routes {
        let mut vec_routes: Routes = vec![];
        for route in self {
            if route.path == path {
                vec_routes.push(route.clone())
            }
        }
        vec_routes
    }

    fn find_route_by_buffer(&self, buffer: &[u8]) -> Routes {
        let mut vec_routes: Routes = vec![];
        for route in self {
            if buffer.starts_with(route.to_complete_string().as_bytes()) {
                vec_routes.push(route.clone())
            }
        }
        vec_routes
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer);

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