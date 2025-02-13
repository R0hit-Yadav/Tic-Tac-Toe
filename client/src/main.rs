// client.rs
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await?;
    println!("Connected to {}", addr);

    let (read_half, write_half) = stream.into_split();
    let mut server_reader = BufReader::new(read_half);
    let mut server_writer = BufWriter::new(write_half);

    let mut stdin_reader = BufReader::new(tokio::io::stdin());
    loop {
        let mut line = String::new();
        let mut input = String::new();
        
        tokio::select! {
            bytes = server_reader.read_line(&mut line) => {
                if let Ok(read_bytes) = bytes {
                    if read_bytes == 0 {
                        println!("Server closed the connection.");
                        break;
                    }
                    println!("{}", line);
                } else {
                    println!("Error reading from server: {:?}", bytes.err());
                    break;
                }
            },
            bytes = stdin_reader.read_line(&mut input) => {
                if let Ok(read_bytes) = bytes {
                    if read_bytes == 0 {
                        println!("Stdin closed.");
                        break;
                    }
                    server_writer.write_all(input.as_bytes()).await?;
                    server_writer.flush().await?;
                } else {
                    println!("Error reading from stdin: {:?}", bytes.err());
                    break;
                }
            }
        }
    }
    Ok(())
}


