use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::select;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:2525").await?;
    println!("Connected to server at 127.0.0.1:2525");

    // Create a BufReader to read incoming messages from the server
    let mut reader = BufReader::new(stream.clone());

    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin);

    let timeout_duration = Duration::from_secs(10); 

    loop {
        select! {
            result = reader.read_line(&mut String::new()) => {
                let mut buffer = String::new();
                match result {
                    Ok(0) => {
                        println!("Disconnected from server.");
                        break;
                    }
                    Ok(_) => {
                        buffer = String::new();
                        reader.read_line(&mut buffer).await?;
                        println!("Server response: {}", buffer.trim());
                    }
                    Err(e) => {
                        println!("Error reading from server: {}", e);
                        break;
                    }
                }
            }


            result = stdin.read_line(&mut String::new()) => {
                let mut input = String::new();
                match result {
                    Ok(_) => {
                        input = String::new();
                        stdin.read_line(&mut input).await?;
                        
                        let trimmed_input = input.trim();
                        if let Err(e) = stream.write_all(trimmed_input.as_bytes()).await {
                            println!("Error writing to server: {}", e);
                            break;
                        }
                        stream.flush().await?;
                    }
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        break;
                    }
                }
            }

            _ = time::sleep(timeout_duration) => {
                println!("Timeout reached, no activity.");
            }
        }
    }

    Ok(())
}
