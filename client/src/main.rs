use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> io::Result<()> 
{
    let stream = TcpStream::connect("127.0.0.1:2525").await?; //connect server
    let (reader, mut writer) = stream.into_split(); // split into reader and writer

    
    let mut reader = BufReader::new(reader);//reader
    let mut line = String::new();

    let game_over_flag = std::sync::Arc::new(tokio::sync::Mutex::new(false));// game over flag 
    let game_over_flag_clone = game_over_flag.clone();

    // Spawn a new asynchronous task to handle user input
    tokio::spawn(async move 
    {
        loop 
        {
            if *game_over_flag_clone.lock().await {
                break; // Stop taking input if the game is over
            }
            
            let mut input = String::new();//take input
            std::io::stdin().read_line(&mut input).unwrap();
            
            writer.write_all(input.as_bytes()).await;// write to server
            
            
            println!("Wait for other Player Move");
        }
    });


    // Main loop to read lines from the TCP stream
    loop 
    {
        line.clear();
        if reader.read_line(&mut line).await.unwrap() == 0 {break}
        println!("{}", line);
    }

    Ok(())
}
