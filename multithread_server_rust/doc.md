# Multithreaded Web Server in Rust

## Creating a Sigle Threaded Server first

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

### 