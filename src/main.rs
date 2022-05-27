use hello_web::{ThreadPool, ThreadPoolStatus};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let status: Arc<Mutex<ThreadPoolStatus>> = Arc::new(Mutex::new(ThreadPoolStatus::Action));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Checking for server status. Shutting down if necessary.
        if let ThreadPoolStatus::Terminate = *status.lock().unwrap() {
            break;
        }

        let st = Arc::clone(&status);
        pool.execute(
            || handle_connection(stream),
            move || {
                *st.lock().unwrap() = ThreadPoolStatus::Terminate;
            },
        );
    }

    println!("Shutting down...");
    // Server will shut down as thread pool is dropped.
}

fn handle_connection(mut stream: TcpStream) -> ThreadPoolStatus {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let shutdown = b"GET /shutdown HTTP/1.1\r\n";
    let styles = b"GET /styles.css HTTP/1.1\r\n";

    let mut resulting_status = ThreadPoolStatus::Action;

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "./front/hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "./front/sleep.html")
    } else if buffer.starts_with(shutdown) {
        resulting_status = ThreadPoolStatus::Terminate;
        ("HTTP/1.1 200 OK", "./front/shutdown.html")
    } else if buffer.starts_with(styles) {
        ("HTTP/1.1 200 OK", "./front/styles.css")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "./front/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    resulting_status
}
