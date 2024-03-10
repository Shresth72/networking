use multithread_server_rust::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
mod mutex;
use mutex::main as mutex_main;

fn main() {
    mutex_main();

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4).unwrap_or_else(|err| {
        eprintln!("Error creating pool: {:?}", err.details);
        std::process::exit(1);
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(line) => match line {
            Ok(line) => Some(line),
            Err(_) => None,
        },
        None => None,
    };

    if let Some(request_line) = request_line {
        let (status_line, filename) = match request_line.as_str() {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "response.json"),
            "GET /sleep HTTP/1.1" => {
                std::thread::sleep(std::time::Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "response.json")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };

        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len();

        let response = format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            handle_file_type(filename),
            length,
            contents
        );

        stream.write(response.as_bytes()).unwrap();
    } else {
        println!("No request line received");
        // TODO: Log error
    }
}

fn handle_file_type(filename: &str) -> &str {
    let file_type = filename.split('.').last().unwrap();
    match file_type {
        "html" => "text/html",
        "json" => "application/json",
        _ => "text/plain",
    }
}
