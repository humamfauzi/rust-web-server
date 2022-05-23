use std::thread;
use std::sync::mpsc;

use std::sync::Arc;
use std::sync::Mutex;

type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads on the pool.
    /// 
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero or less
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender}
    }
    pub fn execute<F>(&self, f: F) 
    where F: FnOnce() + Send + 'static, 
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move|| loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job!", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} terminated", id);
                    break;
                }
            }
        });
        Worker { 
            id, 
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop (&mut self) {
        println!("terminating all workers...");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("shutting down {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Route<'a> {
    version: &'a str,
    path: &'a str,
    method: &'a str,
    reply: &'a str,
    filename: &'a str,
}

impl<'a> Route<'a> {
    pub fn new(version: &'a str, path: &'a str, method: &'a str, reply: &'a str, filename: &'a str) -> Route<'a> {
        Route{ version, path, method, reply, filename, }
    }
    fn to_complete_string(&self) -> String {
        let check = format!("{} {} {}", self.method, self.path, self.version);
        check
    }

    pub fn reply(&self) ->  (&str,  &str) {
        (self.reply, self.filename)
    }
}

pub type Routes<'a> = Vec<Route<'a>>;
pub trait RoutesProperties {
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