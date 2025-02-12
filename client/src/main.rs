use std::net::TcpStream;
use std::io::{self, BufReader, BufRead, Write};
use std::thread;

fn main() -> std::io::Result<()> 
{
    let mut stream = TcpStream::connect("127.0.0.1:2525")?;
    println!("Connected to server at 127.0.0.1:2525");

    // Clone the stream to use in a reader thread.
    let stream_clone = stream.try_clone()?;
    thread::spawn(move || {
        let mut reader = BufReader::new(stream_clone);
        loop {
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("Disconnected from server.");
                    break;
                },
                Ok(_) => {
                    // Print whatever the server sent.
                    print!("{}", buffer);
                },
                Err(e) => {
                    println!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Main thread: read from standard input and send to the server.
    let stdin = io::stdin();
    loop {
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        stream.write_all(input.as_bytes())?;
        stream.flush()?;
    }
}
