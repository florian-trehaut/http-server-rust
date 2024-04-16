use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let buf = BufReader::new(&mut stream);
                let request = buf.lines().next().unwrap().unwrap();
                let status_line = match &request[..] {
                    "GET / HTTP/1.1" => "HTTP/1.1 200 OK \r\n\r\n",
                    _ => "HTTP/1.1 404 NOT FOUND \r\n\r\n",
                };
                let response = status_line;
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
