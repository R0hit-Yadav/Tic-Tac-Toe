use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque; // For the waiting queue.

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

    // Check winner
    fn check_winner(&self, marker: char) -> bool {
        let wins = [[0, 1, 2],[3, 4, 5],[6, 7, 8],[0, 3, 6],[1, 4, 7],[2, 5, 8],[0, 4, 8],[2, 4, 6],];
        for win in wins.iter() {
            if self.cells[win[0]] == marker &&
               self.cells[win[1]] == marker &&
               self.cells[win[2]] == marker {
                return true;
            }
        }
        false
    }

    //check the game is draw
    fn is_draw(&self) -> bool {
        self.cells.iter().all(|&c| c != ' ')
    }
}


// Write a message to the stream and flush it.
fn write_all(stream: &mut TcpStream, msg: &str) -> std::io::Result<()> {
    stream.write_all(msg.as_bytes())?;
    stream.flush()?;
    Ok(())
}


// Handle a game session between two players.
fn handle_game_session(mut stream1: TcpStream,mut stream2: TcpStream,waiting_queue: Arc<Mutex<VecDeque<TcpStream>>>) 
{
    // Create separate readers for each player.
    let mut reader1 = BufReader::new(stream1.try_clone().unwrap());
    let mut reader2 = BufReader::new(stream2.try_clone().unwrap());

    loop 
    {
        let mut board = Board::new();
        let markers = ['⭕', '❌'];
        let mut current = 0; 

        let _ = write_all(&mut stream1, "Game started! You are ⭕\n");
        let _ = write_all(&mut stream2, "Game started! You are ❌\n");

        let _ = write_all(&mut stream1, &board.display()); //print initial board
        let _ = write_all(&mut stream2, &board.display());

        loop // --- Game Round Loop ---
        {
            let prompt = "Make Your Move(1-9): \n";
            
            // let board_disp = board.display();
            // let _ = write_all(&mut stream1, &board_disp);
            // // println!("board: {}\n", board_disp);
            // let _ = write_all(&mut stream2, &board_disp);

            let (current_stream, current_reader) = if current == 0 
            {
                (&mut stream1, &mut reader1)
            } else {
                (&mut stream2, &mut reader2)
            };
            
            let _ = write_all(current_stream, prompt); // Prompt the current player for a move.

            let mut input = String::new();

            if current_reader.read_line(&mut input).is_err()
            {
                let other_stream = if current == 0 { &mut stream2 } else { &mut stream1 };
                let _ = write_all(other_stream, "Opponent disconnected⛔. Waiting for a new opponent👨...\n");
                waiting_queue.lock().unwrap().push_back(other_stream.try_clone().unwrap());
                return;
            }

            let trimmed = input.trim();

            // Check if the input is a valid number between 1 and 9.
            let pos: usize = match trimmed.parse::<usize>() 
            {
                Ok(num) if num >= 1 && num <= 9 => num - 1,
                _ => 
                {
                    let _ = write_all(current_stream, "Invalid input❗. Please enter a number between 1 and 9.\n");
                    continue;
                }
            };

            // check move is taken
            if !board.make_move(pos, markers[current]) {
                let _ = write_all(current_stream, "Invalid move. Cell already taken❌.\n");
                let board_disp = board.display();
                let _ = write_all(current_stream, &board_disp);
                continue;
            }

            // print updated board
            let board_disp = board.display();
            let _ = write_all(&mut stream1, &board_disp);
            let _ = write_all(&mut stream2, &board_disp);
            println!("Game Board: {}\n", board_disp);


            // Check if the current player has won or if the game is a draw.
            if board.check_winner(markers[current]) {
                let win_msg = format!("Player {} wins!\n", if current == 0 { "⭕" } else { "❌" });
                let _ = write_all(&mut stream1, &win_msg);
                let _ = write_all(&mut stream2, &win_msg);

                println!("Final Board: {}\n", board_disp);
                println!("Player {} Wins!",win_msg);
                break;
            } else if board.is_draw() {
                let _ = write_all(&mut stream1, "It's a Draw!🤝\n");
                let _ = write_all(&mut stream2, "It's a Draw!🤝\n");

                println!("Final Board: {}\n", board_disp);
                println!("It's a Draw!");
                break;
            }

            current = 1 - current; // Switch players.
        } // --- End of Game Round Loop ---

        // Ask both players if they want to play again.
        loop 
        {
            let _ = write_all(&mut stream1, "Play Again?🤔 (Y/N): \n");
            let _ = write_all(&mut stream2, "Play Again?🤔 (Y/N): \n");
            
            let mut response1 = String::new();
            let mut response2 = String::new();

            if reader1.read_line(&mut response1).is_err() 
            {
            let _ = write_all(&mut stream2, "Opponent disconnected⛔.\n");
            waiting_queue.lock().unwrap().push_back(stream2.try_clone().unwrap());
            return;
            }

            if reader2.read_line(&mut response2).is_err() 
            {
            let _ = write_all(&mut stream1, "Opponent disconnected⛔.\n");
            waiting_queue.lock().unwrap().push_back(stream1.try_clone().unwrap());
            return;
            }

            let response1 = response1.trim().to_lowercase();
            let response2 = response2.trim().to_lowercase();

            if (response1 == "y" || response1 == "n") && (response2 == "y" || response2 == "n") 
            {
                if response1 == "y" && response2 == "y" {
                    let _ = write_all(&mut stream1, "Restarting game🔃...\n");
                    let _ = write_all(&mut stream2, "Restarting game🔃...\n");
                    break; // start a new game round with the same pair
                } else {
                    // If one player does not wish to continue, inform both.
                    if response1 != "y" {
                    let _ = write_all(&mut stream1, "You Have Chose to Quit. GoodBye!👋\n");
                    let _ = write_all(&mut stream2, "Opponent Has Quit. Waiting for New Opponent to Join...\n");
                    waiting_queue.lock().unwrap().push_back(stream2.try_clone().unwrap());
                    }
                    if response2 != "y" {
                    let _ = write_all(&mut stream2, "You Have Chose to Quit. GoodBye!👋\n");
                    let _ = write_all(&mut stream1, "Opponent Has Quit. Waiting for New Opponent to Join...\n");
                    waiting_queue.lock().unwrap().push_back(stream1.try_clone().unwrap());
                    }
                    return;
                }
            }
            else 
            {
                let _ = write_all(&mut stream1, "Invalid input❗. Please enter 'Y' or 'N'.\n");
                let _ = write_all(&mut stream2, "Invalid input❗. Please enter 'Y' or 'N'.\n");
            }
        }
    }
}


// Handle a new connection.
fn handle_new_connection(mut stream: TcpStream, waiting_queue: Arc<Mutex<VecDeque<TcpStream>>>)
{
    let _ = write_all(&mut stream, "Welcome to ⭕Tic Tac Toe!❌ Waiting for an opponent...\n");
    let mut queue = waiting_queue.lock().unwrap(); // Lock the waiting queue.
    if let Some(opponent) = queue.pop_front() 
    {
        let waiting_queue_clone = Arc::clone(&waiting_queue);
        let stream_clone = stream.try_clone().unwrap();

        thread::spawn(move || { // Start a new thread for the game session.
            handle_game_session(opponent, stream_clone, waiting_queue_clone);
        });
    } 
    else 
    {
        queue.push_back(stream); 
    }
}

fn main() -> std::io::Result<()> {

    let listener = TcpListener::bind("0.0.0.0:2525")?;
    println!("Server listening on port 2525");
 

    // Create a waiting queue to store incoming connections.
    let waiting_queue = Arc::new(Mutex::new(VecDeque::new()));

    for stream in listener.incoming() 
    {
        match stream 
        {
            Ok(stream) => {
                println!("New connection");
                let waiting_queue_clone = Arc::clone(&waiting_queue);
                thread::spawn(move || {
                    handle_new_connection(stream, waiting_queue_clone);
                });
            },
            Err(e) => 
            {
                println!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
