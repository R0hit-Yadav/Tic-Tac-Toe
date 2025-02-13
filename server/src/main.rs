// server.rs
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    // A shared waiting queue for clients
    let waiting = Arc::new(Mutex::new(Vec::new()));

    loop 
    {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {:?}", addr);

        let client = setup_client(socket).await;
        {
            let mut queue = waiting.lock().await;
            queue.push(client);
        }

        // If at least two clients are waiting, pair them and start a game session.
        let mut pair = None;
        {
            let mut queue = waiting.lock().await;
            if queue.len() >= 2 {
                let client1 = queue.remove(0);
                let client2 = queue.remove(0);
                pair = Some((client1, client2));
            }
        }
        if let Some((client1, client2)) = pair {
            let waiting_clone = waiting.clone();
            tokio::spawn(async move {
                game_session(client1, client2, waiting_clone).await;
            });
        }
    }
}


type Rx = mpsc::Receiver<String>;
//a waiting client
struct Client {
    rx: Rx,
    writer: BufWriter<tokio::net::tcp::OwnedWriteHalf>,
}

// A client that is currently in a game (with a symbol assigned)
struct Player {
    symbol: char,
    rx: Rx,
    writer: BufWriter<tokio::net::tcp::OwnedWriteHalf>,
}

// Allow converting a Player (finished game session) back to a Client.
impl From<Player> for Client {
    fn from(player: Player) -> Self {
        Client {
            rx: player.rx,
            writer: player.writer,
        }
    }
}


/// Given a TcpStream, split it into a reader and writer, and spawn a task that reads lines from the client and sends them on a channel.
async fn setup_client(socket: TcpStream) -> Client 
{
    let (read_half, write_half) = socket.into_split();

    let reader = BufReader::new(read_half);
    let writer = BufWriter::new(write_half);

    // Create an mpsc channel for lines read from the client.
    let (tx, rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await 
        {
            // If sending fails (e.g. the receiver was dropped), exit the loop.
            if tx.send(line).await.is_err() {
                break;
            }
        }
    });

    Client { rx, writer }
}


fn format_board(board: &[Option<char>; 9]) -> String {
    let mut s = String::new();
    for i in 0..9 {
        let cell = match board[i] {
            Some(c) => c.to_string(),
            None => (i + 1).to_string(),
        };
        s.push_str(&cell);
        if (i + 1) % 3 == 0 {
            if i != 8 {
                s.push_str("\n- + - + - \n");
            }
        } else {
            s.push_str(" | ");
        }
    }
    s
}


fn check_winner(board: &[Option<char>; 9]) -> Option<char> {
    let lines = [
        (0, 1, 2),
        (3, 4, 5),
        (6, 7, 8),
        (0, 3, 6),
        (1, 4, 7),
        (2, 5, 8),
        (0, 4, 8),
        (2, 4, 6),
    ];
    for &(a, b, c) in &lines {
        if let (Some(x), Some(y), Some(z)) = (board[a], board[b], board[c]) {
            if x == y && y == z {
                return Some(x);
            }
        }
    }
    None
}


async fn game_session(client1: Client, client2: Client, waiting: Arc<Mutex<Vec<Client>>>) {

    // Assign symbols: player1 gets 'X', player2 gets 'O'
    let mut player1 = Player {
        symbol: '❌',
        rx: client1.rx,
        writer: client1.writer,
    };
    let mut player2 = Player {
        symbol: '⭕',
        rx: client2.rx,
        writer: client2.writer,
    };

    // Outer loop to allow replaying games.
    'game_session: loop {
        let mut board: [Option<char>; 9] = [None; 9];
        let mut current_turn = 0; // 0: player1’s turn, 1: player2’s turn

        // Game loop: repeat until there’s a win or tie.
        loop {
            // Send the updated board to both players.
            let board_str = format_board(&board);
            let msg = format!("{}\n", board_str);
            
            let _ = player1.writer.write_all(msg.as_bytes()).await;
            let _ = player1.writer.write_all(b"\n").await;
            let _ = player1.writer.flush().await;

            let _ = player2.writer.write_all(msg.as_bytes()).await;
            let _ = player2.writer.write_all(b"\n").await;
            let _ = player2.writer.flush().await;

   
            let prompt = format!(
                "Player {} ({}) - Enter your move (1-9): \n",
                if current_turn == 0 { "1" } else { "2" },
                if current_turn == 0 { player1.symbol } else { player2.symbol }
            );
            if current_turn == 0 {
                let _ = player1.writer.write_all(prompt.as_bytes()).await;
                let _ = player1.writer.flush().await;

            } else {
                let _ = player2.writer.write_all(prompt.as_bytes()).await;
                let _ = player2.writer.flush().await;
                
            }

            // Here we use a loop with tokio::select! to wait for input. Only the player whose turn it is will have their input accepted;
            // if the other sends input, they get a “Not your turn” message.
            let move_str = loop {
                tokio::select! {
                    line = player1.rx.recv() => {
                        if current_turn == 0 {
                            break line;
                        } else {
                            if let Some(_msg) = line {
                                let _ = player1.writer.write_all(b"Not your turn.\n").await;
                                let _ = player1.writer.flush().await;
                            } else {
                                break None;
                            }
                        }
                    },
                    line = player2.rx.recv() => {
                        if current_turn == 1 {
                            break line;
                        } else {
                            if let Some(_msg) = line {
                                let _ = player2.writer.write_all(b"Not your turn.\n").await;
                                let _ = player2.writer.flush().await;
                            } else {
                                break None;
                            }
                        }
                    },
                }
            };

            // If a client disconnects (None received), inform the other and end the game.
            let move_str = if let Some(m) = move_str {
                m
            } else {
                if current_turn == 0 {
                    let _ = player2.writer.write_all(b"Opponent disconnected. You win by default.\n").await;
                    let _ = player2.writer.flush().await;
                } else {
                    let _ = player1.writer.write_all(b"Opponent disconnected. You win by default.\n").await;
                    let _ = player1.writer.flush().await;
                }
                break;
            };

            // Parse the move: expect a number between 1 and 9.
            let pos: usize = match move_str.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= 9 => n - 1,
                _ => {
                    let err_msg = b"Invalid move. Please enter a number between 1 and 9.\n";
                    if current_turn == 0 {
                        let _ = player1.writer.write_all(err_msg).await;
                        let _ = player1.writer.flush().await;
                    } else {
                        let _ = player2.writer.write_all(err_msg).await;
                        let _ = player2.writer.flush().await;
                    }
                    continue;
                }
            };

            // Reject moves that target an already taken cell.
            if board[pos].is_some() {
                let err_msg = b"Cell already taken. Try again.\n";
                if current_turn == 0 {
                    let _ = player1.writer.write_all(err_msg).await;
                    let _ = player1.writer.flush().await;
                } else {
                    let _ = player2.writer.write_all(err_msg).await;
                    let _ = player2.writer.flush().await;
                }
                continue;
            }

            // Update the board.
            if current_turn == 0 {
                board[pos] = Some(player1.symbol);
            } else {
                board[pos] = Some(player2.symbol);
            }

            // Check for a win.
            if let Some(winner) = check_winner(&board) {
                let board_str = format_board(&board);
                let win_msg = format!("{}\n\nPlayer {} ({}) wins!\n", board_str, if current_turn == 0 { "1" } else { "2" }, winner);
                let _ = player1.writer.write_all(win_msg.as_bytes()).await;
                let _ = player1.writer.flush().await;
                let _ = player2.writer.write_all(win_msg.as_bytes()).await;
                let _ = player2.writer.flush().await;
                break;
            }

            // Check for a tie.
            if board.iter().all(|&cell| cell.is_some()) {
                let board_str = format_board(&board);
                let tie_msg = format!("{}\n\nIt's a tie!\n", board_str);
                let _ = player1.writer.write_all(tie_msg.as_bytes()).await;
                let _ = player1.writer.flush().await;
                let _ = player2.writer.write_all(tie_msg.as_bytes()).await;
                let _ = player2.writer.flush().await;
                break;
            }

            // Switch turns.
            current_turn = 1 - current_turn;

        } // end of a single game

        // Ask both players if they want to play again.
        let replay_prompt = b"\nPlay again? (yes/no): \n";
        let _ = player1.writer.write_all(replay_prompt).await;
        let _ = player1.writer.flush().await;
        let _ = player2.writer.write_all(replay_prompt).await;
        let _ = player2.writer.flush().await;

        // join Wait for responses concurrently.
        let (resp1, resp2) = tokio::join!(player1.rx.recv(), player2.rx.recv());

        let play1 = resp1.unwrap_or_else(|| "no".to_string()).trim().to_lowercase() == "yes";
        let play2 = resp2.unwrap_or_else(|| "no".to_string()).trim().to_lowercase() == "yes";

        if play1 && play2 {
            let restart_msg = b"\nRestarting game...\n";
            let _ = player1.writer.write_all(restart_msg).await;
            let _ = player1.writer.flush().await;
            let _ = player2.writer.write_all(restart_msg).await;
            let _ = player2.writer.flush().await;
            continue 'game_session;

        } else if play1 && !play2 {
            let msg = b"Opponent left. Waiting for a new opponent...\n";
            let _ = player1.writer.write_all(msg).await;
            let _ = player1.writer.flush().await;
            let client: Client = player1.into();
            {
                let mut queue = waiting.lock().await;
                queue.push(client);
            }
            break;

        } else if !play1 && play2 {
            let msg = b"Opponent left. Waiting for a new opponent...\n";
            let _ = player2.writer.write_all(msg).await;
            let _ = player2.writer.flush().await;
            let client: Client = player2.into();
            {
                let mut queue = waiting.lock().await;
                queue.push(client);
            }
            break;
        } else {
            let bye = b"Goodbye!\n";
            let _ = player1.writer.write_all(bye).await;
            let _ = player1.writer.flush().await;
            let _ = player2.writer.write_all(bye).await;
            let _ = player2.writer.flush().await;
            break;
        }
    } // end of game_session loop
}










