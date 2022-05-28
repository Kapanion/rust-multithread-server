# Multithreaded Web Server Made with Rust

The server listens for TCP connections at address `127.0.0.1:7878`. Several pages can be accessed:

- `127.0.0.1:7878` - Default page, simple HTML.
- `127.0.0.1:7878/sleep` - Server responds with a 5 second delay. This is made for testing its multithreaded aspect.
- `127.0.0.1:7878/shutdown` - Asks for the password, and if it's correctly provided, shuts the server down.
- `127.0.0.1:7878/[non-existent link]` - Gives us the "404 Not Found" page.

## How to Run

To run this project, you must have `cargo` installed. Open the base directory and run the following command:
```
cargo run
```
The server can be shut down either by pressing `Ctrl + C` in the terminal or visiting the shutdown page and providing the correct password.

## Implementation Details

This project is partially based on the final project from [The Rust Programming Language](https://doc.rust-lang.org/book/) book.

Multithreading is done with thread pool. The code is split in two files: __main.rs__ and __lib.rs__. The name of the crate is `hello_web`.
- __lib.rs__ contains the implementation of the public `ThreadPool` struct, which manages the worker threads, defined by the `Worker` struct.
- __main.rs__ contains the code responsible for getting HTTP requests and sending appropriate responses with the `handle_connection` function.

Each worker thread receives the `handle_connection` function as well as the callback function that should be called for shutting down the server. Each job (or task) given to the worker threads returns the `ThreadPoolStatus` enum which tells them whether the server needs to be shut down after completing the job.