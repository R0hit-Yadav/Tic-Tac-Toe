use tokio::net::{TcpListener, TcpStream}; // TcpListener and TcpStream for TCP networking
use tokio::sync::Mutex; // for synchronization
use std::sync::Arc; // for reference counting
use std::io::{self}; 
use tokio::io::{AsyncBufReadExt, AsyncWriteExt}; // for asynchronous I/O operations


#[tokio::main]
async fn main() -> io::Result<()> 
{
    
    let listener = TcpListener::bind("127.0.0.1:2525").await?; // Bind the TCP listener to the address
    let state = Arc::new(Mutex::new(GameState::new()));  // Create a shared game state
    let mut player_count = 0;

    println!("Server is running on 127.0.0.1:2525");


    // Accept incoming connections
    while let Ok((stream, _)) = listener.accept().await 
    {
        player_count += 1;
        // chacked for two players
        if player_count > 2 
        {
            println!("Only two players can play.");
            break;
        }

        let state_clone = state.clone();

        // Spawn a new task to handle the player
        tokio::spawn(async move {handle_player(stream, state_clone, player_count).await});
    }
    Ok(())
}

#[derive(Clone)]
struct GameState 
{
    board: Vec<String>, // game board 
    current_player: usize, // whose turn it is 
    game_over: bool, // is the game over
}

impl GameState 
{
    fn new() -> Self 
    {
        GameState 
        {
            board: vec![" ".to_string(); 9], // the board with empty cells
            current_player: 1, // player 1 starts the game
            game_over: false, // the game is not over yet
        }
    }

    // display number in bord
    fn board_num_display(&self, index: usize) -> String 
    {
        if self.board[index] == " " 
        {
            index.to_string() 
        } 
        else 
        {
            self.board[index].clone() 
        }
    }


    // Format the board  for display
    fn display_board(&self) -> String 
    {
        format!("\n---------------\n {} | {} | {}\n___-___-___\n {} | {} | {}\n___-___-___\n {} | {} | {}\n---------------",
        self.board_num_display(0),self.board_num_display(1),self.board_num_display(2),self.board_num_display(3),self.board_num_display(4),self.board_num_display(5),self.board_num_display(6),self.board_num_display(7),self.board_num_display(8))
    }


    // for a move on the board
    fn make_move(&mut self, position: usize, symbol: &str) -> Result<(), String>
    {
        if position >= 9 || self.board[position] != " " 
        {
            return Err("Invalid move. position is already occupied or it's not there".to_string());
        }
        self.board[position] = symbol.to_string();

        println!("Updated Board:\n{}", self.display_board());
        Ok(())
    }

    // Check winnner or draw
    fn winner_chacking(&self) -> Option<String> 
    {
        let win = [[0, 1, 2],[3, 4, 5],[6, 7, 8],[0, 3, 6],[1, 4, 7],[2, 5, 8],[0, 4, 8],[2, 4, 6]];

        for combo in win.iter() 
        {
            if self.board[combo[0]] != " " && self.board[combo[0]] == self.board[combo[1]] && self.board[combo[1]] == self.board[combo[2]] 
            {
                return Some(self.board[combo[0]].clone());
            }
        }

        // If all cells are occupied, it's a draw
        if self.board.iter().all(|cell| cell != " ") 
        {
            return Some("Draw".to_string());
        }
        None
    }
}

async fn handle_player(stream: TcpStream, state: Arc<Mutex<GameState>>, player_id: usize) 
{
    // split the stream into reader and writer
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = tokio::io::BufReader::new(reader);

    // give symbols to players
    let symbol = if player_id == 1 {"🅾️"} else {"❎"};

    writer.write_all(format!("You are Player {} ({})\n", player_id, symbol).as_bytes()).await.unwrap();

    loop 
    {
        let mut state = state.lock().await; // Lock the game state for the current player

        if state.game_over {
            break;
        }

        if state.current_player != player_id 
        {
            continue; // Wait for your turn
        }

        writer.write_all(format!("Current Board: {}\n", state.display_board()).as_bytes()).await.unwrap();
        writer.write_all("Make Move From (0-8): \n".as_bytes()).await.unwrap();

        let mut input = String::new();

        reader.read_line(&mut input).await.unwrap();
        let position: usize = match input.trim().parse() 
        {
            Ok(pos) => pos,
            Err(_) => 
            {
                writer.write_all("Invalid input. Try again.\n".as_bytes()).await.unwrap();
                continue;
            }
        };

        match state.make_move(position, symbol) 
        {
            Ok(_) => 
            {
                state.current_player = if player_id == 1 { 2 } else { 1 }; // Switch turn to the other player
                if let Some(winner) = state.winner_chacking() 
                {
                    let message = if winner == "Draw" 
                    {
                        "Game Over! It's a Draw 🤝".to_string()
                    } 
                    else 
                    {
                        format!("Game Over! Winner: {}", winner)
                    };

                    
                    
                    // Notify both players about the game result
                    writer.write_all(format!("{}\nGame Finish\n", message).as_bytes()).await.unwrap();
                    
                    println!("{}", message); // Print the result on the server console as well

                    
                    state.game_over = true;

                    // std::process::exit(0); // Exit the program
                    break;
                    

                }
            }
            Err(_) => 
            {
                writer.write_all(format!("Game IS Finish\n").as_bytes()).await.unwrap();
            }
        }
    }
}

