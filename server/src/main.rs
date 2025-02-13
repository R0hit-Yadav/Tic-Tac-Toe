use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::collections::VecDeque;


struct Board { // Tic Tac Toe Board
    cells: [char; 9],
}

impl Board { 
    fn new() -> Board {
        Board { cells: [' '; 9] }
    }

    fn display(&self) -> String { // Display the board
        let mut s = String::new();
        s.push_str("\n-------------\n");
        for i in 0..3 {
            s.push_str("| ");
            for j in 0..3 {
                let index = i * 3 + j;
                if self.cells[index] == ' ' {
                    s.push_str(&format!("{} ", index + 1));
                } else {
                    s.push(self.cells[index]);
                    s.push(' ');
                }
                s.push_str("| ");
            }
            s.push_str("\n-------------\n");
        }
        s
    }

    fn make_move(&mut self, pos: usize, marker: char) -> bool { // Make a move
        if pos < 9 && self.cells[pos] == ' ' {
            self.cells[pos] = marker;
            true
        } else {
            false
        }
    }

    fn check_winner(&self, marker: char) -> bool {
        let wins = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];
        for win in wins.iter() {
            if self.cells[win[0]] == marker && self.cells[win[1]] == marker && self.cells[win[2]] == marker {
                return true;
            }
        }
        false
    }

    fn is_draw(&self) -> bool {
        self.cells.iter().all(|&c| c != ' ')
    }
}

async fn write_all(stream: &mut TcpStream, msg: &str) -> std::io::Result<()> {
    stream.write_all(msg.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn handle_game_session(mut stream1: TcpStream, mut stream2: TcpStream, waiting_queue: Arc<Mutex<VecDeque<TcpStream>>>) {
    let mut reader1 = BufReader::new(stream1);
    let mut reader2 = BufReader::new(stream2);


        let mut board = Board::new();
        let markers = ['⭕', '❌'];
        let mut current = 0;

        let _ = write_all(&mut stream1, "Game started! You are ⭕\n").await;
        let _ = write_all(&mut stream2, "Game started! You are ❌\n").await;

        let _ = write_all(&mut stream1, &board.display()).await; // print initial board
        let _ = write_all(&mut stream2, &board.display()).await;

        loop {
            let prompt = "Make Your Move(1-9): \n";
            let (current_stream, current_reader) = if current == 0 {
                (&mut stream1, &mut reader1)
            } else {
                (&mut stream2, &mut reader2)
            };

            let _ = write_all(current_stream, prompt).await; // Prompt the current player for a move.
            let mut input = String::new();

            tokio::select! {
                result = current_reader.read_line(&mut input) => {
                    if result.is_err() {
                        let other_stream = if current == 0 { &mut stream2 } else { &mut stream1 };
                        let _ = write_all(other_stream, "Opponent disconnected⛔. Waiting for a new opponent👨...\n").await;
                        waiting_queue.lock().await.push_back(other_stream.try_clone().unwrap());
                        return;
                    }
                }
                _ = sleep(Duration::from_secs(10)) => {
                    let _ = write_all(current_stream, "Timeout: You took too long! Waiting for the next move.\n").await;
                    continue;
                }
            }

            let trimmed = input.trim();
            let pos: usize = match trimmed.parse::<usize>() {
                Ok(num) if num >= 1 && num <= 9 => num - 1,
                _ => {
                    let _ = write_all(current_stream, "Invalid input❗. Please enter a number between 1 and 9.\n").await;
                    continue;
                }
            };

            if !board.make_move(pos, markers[current]) {
                let _ = write_all(current_stream, "Invalid move. Cell already taken❌.\n").await;
                continue;
            }

            let board_disp = board.display();
            let _ = write_all(&mut stream1, &board_disp).await;
            let _ = write_all(&mut stream2, &board_disp).await;

            println!("Game Board: {}\n", board_disp);

            if board.check_winner(markers[current]) {
                let win_msg = format!("Player {} wins!\n", if current == 0 { "⭕" } else { "❌" });
                let _ = write_all(&mut stream1, &win_msg).await;
                let _ = write_all(&mut stream2, &win_msg).await;
                break;
            } else if board.is_draw() {
                let _ = write_all(&mut stream1, "It's a Draw!🤝\n").await;
                let _ = write_all(&mut stream2, "It's a Draw!🤝\n").await;
                break;
            }

            current = 1 - current; // Switch players.
        }

        loop {
            let _ = write_all(&mut stream1, "Play Again?🤔 (Y/N): \n").await;
            let _ = write_all(&mut stream2, "Play Again?🤔 (Y/N): \n").await;

            let mut response1 = String::new();
            let mut response2 = String::new();

            tokio::select! {
                result1 = reader1.read_line(&mut response1) => {
                    if result1.is_err() {
                        let _ = write_all(&mut stream2, "Opponent disconnected⛔.\n").await;
                        waiting_queue.lock().await.push_back(stream2.try_clone().unwrap());
                        return;
                    }
                }
                result2 = reader2.read_line(&mut response2) => {
                    if result2.is_err() {
                        let _ = write_all(&mut stream1, "Opponent disconnected⛔.\n").await;
                        waiting_queue.lock().await.push_back(stream1.try_clone().unwrap());
                        return;
                    }
                }
            }

            let response1 = response1.trim().to_lowercase();
            let response2 = response2.trim().to_lowercase();

            if (response1 == "y" || response1 == "n") && (response2 == "y" || response2 == "n") {
                if response1 == "y" && response2 == "y" {
                    let _ = write_all(&mut stream1, "Restarting game🔃...\n").await;
                    let _ = write_all(&mut stream2, "Restarting game🔃...\n").await;
                    break;
                } else {
                    if response1 != "y" {
                        let _ = write_all(&mut stream1, "You Have Chose to Quit. GoodBye!👋\n").await;
                        let _ = write_all(&mut stream2, "Opponent Has Quit. Waiting for New Opponent to Join...\n").await;
                        waiting_queue.lock().await.push_back(stream2.try_clone().unwrap());
                    }
                    if response2 != "y" {
                        let _ = write_all(&mut stream2, "You Have Chose to Quit. GoodBye!👋\n").await;
                        let _ = write_all(&mut stream1, "Opponent Has Quit. Waiting for New Opponent to Join...\n").await;
                        waiting_queue.lock().await.push_back(stream1.try_clone().unwrap());
                    }
                    return;
                }
            } else {
                let _ = write_all(&mut stream1, "Invalid input❗. Please enter 'Y' or 'N'.\n").await;
                let _ = write_all(&mut stream2, "Invalid input❗. Please enter 'Y' or 'N'.\n").await;
            }
        }
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let waiting_queue = Arc::new(Mutex::new(VecDeque::new()));

    loop {
        let (stream1, _) = listener.accept().await?;
        let (stream2, _) = listener.accept().await?;

        let queue = waiting_queue.clone();
        tokio::spawn(async move {
            handle_game_session(stream1, stream2, queue).await;
        });
    }
}