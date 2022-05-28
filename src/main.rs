use hello_web::{ThreadPool, ThreadPoolStatus};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

const ADDRESS: &str = "127.0.0.1:7878";
const NUMBER_OF_THREADS: usize = 4;
const SHUTDOWN_PASSWORD: &str = "rust7878";

fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    let pool = ThreadPool::new(NUMBER_OF_THREADS);
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

const GET_HELLO: &[u8; 16] = b"GET / HTTP/1.1\r\n";
const GET_SLEEP: &[u8; 21] = b"GET /sleep HTTP/1.1\r\n";
const GET_CSS: &[u8; 26] = b"GET /styles.css HTTP/1.1\r\n";
const SHUTDOWN_REQ: &[u8; 24] = b"GET /shutdown HTTP/1.1\r\n";
const SHUTDOWN_POST: &[u8; 25] = b"POST /shutdown HTTP/1.1\r\n";
const SHUTDOWN: &[u8; 29] = b"GET /totalshutdown HTTP/1.1\r\n";

fn handle_connection(mut stream: TcpStream) -> ThreadPoolStatus {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let (status_line, filename) = if buffer.starts_with(GET_HELLO) {
        ("HTTP/1.1 200 OK", "./front/hello.html")
    } else if buffer.starts_with(GET_SLEEP) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "./front/sleep.html")
    } else if buffer.starts_with(SHUTDOWN_REQ) {
        ("HTTP/1.1 200 OK", "./front/shutdown.html")
    } else if buffer.starts_with(GET_CSS) {
        ("HTTP/1.1 200 OK", "./front/styles.css")
    } else if buffer.starts_with(SHUTDOWN_POST) {
        let pass = "password=".to_owned() + SHUTDOWN_PASSWORD;
        let request_str = String::from_utf8_lossy(&buffer[..]);
        if request_str.contains(&pass) {
            println!("Correct password received. The server will shut down shortly.");
            ("HTTP/1.1 200 OK", "./front/shutdown_successful.html")
        } else {
            println!("Wrong password received.");
            ("HTTP/1.1 200 OK", "./front/shutdown.html")
        }
    } else if buffer.starts_with(SHUTDOWN) {
        return ThreadPoolStatus::Terminate;
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

    ThreadPoolStatus::Action
}
