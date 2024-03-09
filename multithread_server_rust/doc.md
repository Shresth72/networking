# Multi-Threaded Web Server in Rust

## Creating a Single Threaded Server first

### Listening to the TCP Connection

- Binding a port to the TCP listener and listening to the incoming streams.
- The `bind` function returns a `Result<T, E>` which indicates it's possible for binding to fail.
- The `incoming` method on the `TcpListener` returns an iterator that gives us a sequence of the streams (`TcpStream`).
- A single stream represents an open connection between the client and the serve, i.e., the full request and response process between them.
- So, we can read from the `TcpStream` to see what the client sent and then write our response to the stream to send data back to the client.

### Reading the Request

- To implement reading of the requests, we can separate the concerns of first getting a connection and then taking some action, using a new function `handle_connection`.
- It will read data from the TCP stream and print it so we can see the data being sent from the browser.
- `BufReader` adds buffering by managing calls to the `std::io::Read` trait methods for us.
- The `http_request` variable collects the lines of the request inside a vector. The `line` method returns an iterator of `Result<String, Error>`, so we map and unwrap each result.
- After running the code, we can see why we get multiple connections from one browser request by looking at the path after `GET`.
- If the repeated connections are all requesting `/`, the browser is hence trying to fetch `/` repeatedly because it's not getting any response.
- The HTTP Request follows this format

```bash
Method Request-URI HTTP-Version CRLF
headers CRLF
message-body
```

### Writing a response

- To implement sending data in response to the client request, it must follow the following format:

```bash
HTTP-Version Status-Code Reason-Phrase CRLF
headers CRLF
message-body
```

- An example of a HTTP response would be

```bash
HTTP/1.1 200 OK\r\n\r\n
```

### Returning HTML/JSON as response

- Create an html file and read this file in the `handle_connection` function, or specify content-type in headers for JSON data.
- Currently, we are ignoring the request data in the `http_request` and just sending back an html/JSON.

```rust
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 200 OK";
    let json_response = r#"{"message": "Hello, world!"}"#;
    let length = json_response.len();
    let response = format!(
        "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{json_response}"
    );

    stream.write_all(response.as_bytes()).unwrap();
}
```

### Validating the Request and Selectively Responding

- We can add conditional statements to check for content of the request and respond accordingly.
- Also, we use `next()` to only get the first item from the iterator. If the client closes connection before sending any request or there no more lines to read from the stream, we can handle the `None` result and return in the `else` block or log it.
- So, now we can handle all requests to different `/*` routes conditionally.

## Turning Single-Threaded Server into a Multi-Threaded Server

### Simulating a Slow Request in the Current Server Implementation

- 