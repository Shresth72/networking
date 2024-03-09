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

### Improving Throughput with a Thread Pool

- A thread pool is a group of spawned threads that are waiting and ready to handle a task. When the program receives a new task, it assigns one of the threads in the pool to the task, and that thread will process the task.
- The remaining threads in the pool are available to handle any other tasks that come in while the first thread is processing.
- When the first thread is done processing, it's returned to the pool of the idle threads
- This allows for processing connections concurrently, increasing the throughput of the server.
- We also have to limit the number of threads in the pool to avoid DoS attacks.
- The pool will maintain a queue of incoming requests, each of the threads in the pool will pop off a request from this queue, handle the request and ask the queue for another request.

### Spawning a thread for each request

- We can create new threads using `thread::spawn`, so if we open the `/` and `/sleep` requests, simultaneously, the other requests don't have to wait for the `/sleep` to finish.

```rust
fn main() {
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

  for stream in listener.incoming() {
    let stream = stream.unwrap();

    thread::spawn(|| {
      handle_connection(stream);
    });
  }
}
```

### Building ThreadPool using Compiler Driven Development

- We use `ThreadPool::new` to create a new thread pool with a configurable finite number of threads. Then, we can `pool.execute` has a similar interface as `thread::spawn` to run each stream.
- We can create a `ThreadPool` struct and implement the `new` and `execute` methods.
- The `execute` method should take closures as parameter with three traits: `Fn`, `FnMut` and `FnOnce`, and it needs to be similar to `thread::spawn` implementation.

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
  where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

- The `spawn` method uses `FnOnce` as the trait bound on `F`.
- The `F` has trait bound `Send` to transfer the closure from one thread to another and `'static` because we don't know how long the thread will take to execute.

```rust
impl ThreadPool {
  pub fn execute <F>($self, f: F)
  where
    F: FnOnce() + Send + 'static,
    {
    }
}
```

- We still use the `()` after `FnOnce` because this `FnOnce` represents a closure that takes no parameters and returns the unit type `()`.
- We can also, implement a `Worker` that manages the threads, along with an worker `id`. The `ThreadPool` can now store a vector of these `Worker`

### Sending Requests to Threads via Channels

- Currently, the closures given to `thread::spawn` do nothing. We want the `Worker` structs to fetch the code to run from a queue held in the `ThreadPool` and send that code to its thread to run.
- So, to implement this we can:
  - The `ThreadPool` will create a channel and hold on to the sender.
  - Each `Worker` will hold on to the receiver.
  - We'll create a new `Job` struct that will hold the closures we want to send down the channel.
  - The `execute` method will send the job it wants to execute through the sender.
  - In its thread, the `Worker` will loop over its receiver and execute the closures of any jobs it receives.
- To share let multiple workers own the receiver, we use the `Arc` type and `Mutex` to ensure that only one worker gets a job from the receiver at a time.

### Implementing the execute method

- We can change the `Job` struct to a type alias for a trait object that holds the type of closure that `execute` receives.
- After creating `Job` instance using the closure we get in `execute`, we send that job down the sending end of the channel using `send` method.
- But at the moment, the threads continue executing as long as the pool exists and we can't stop them.
- Also, the closure still only references the receiving end of the channel. Instead, we need the closure to loop forever, asking the receiver end of the channel for a job.

```rust
impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread = thread::spawn(move || loop {
    let job = receiver.lock().unwrap().recv().unwrap();

    println!("Worker {id} got a job; executing.");

    job();
  });

    Worker { id, thread }
  }
}
```

- The `lock` is used to acquire the mutex on the `receiver`
- Then `recv` is used to receive a `Job` from the channel. If there is no job yet, the current thread will wait until a job becomes available. The `Mutex<T>` ensures that only one `Worker` thread at a time is trying to request a job.
