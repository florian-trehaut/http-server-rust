# Simple educational server

I reimplemented some of the HTTP protocol to improve myself in Rust. Beware, this is messy.
This is a simple server implemented in Rust. It accepts a command line argument `--directory` to specify the directory where it can read and write files.

## Getting Started

To start the server, navigate to the directory containing the `main.rs` file and run the following command:

```bash
cargo run -- --directory /path/to/your/directory
```

Replace `/path/to/your/directory` with the path to the directory you want the server to read and write files.

## Endpoints

The server implements the following endpoints:

- `GET /`: Returns a welcome message.
- `GET /echo/<string>`: Returns the string that you provide.
- `GET /files/`: Returns the content of file in the directory specified when starting the server.
- `POST /files/`: Writes the request body to a new file in the directory specified when starting the server.

