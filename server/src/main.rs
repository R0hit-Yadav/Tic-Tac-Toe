use tokio::net::{TcpListener, TcpStream}; // TcpListener and TcpStream for TCP networking
use tokio::sync::Mutex; // for synchronization
use std::sync::Arc; // for reference counting
use std::collections::HashMap;
use std::io::{self}; 
use tokio::io::{AsyncBufReadExt, AsyncWriteExt}; // for asynchronous I/O operations


#[tokio::main]
async fn main() -> io::Result<()> 
{
    
    let listener = TcpListener::bind("127.0.0.1:2525").await?; // Bind the TCP listener to the address
    let games: Arc<Mutex<HashMap<usize, GameState>>> = Arc::new(Mutex::new(HashMap::new()));// for store multipl games with game_id
    let mut player_queue = Vec::new();

    println!("Server is running on 127.0.0.1:2525");


    // Accept incoming connections
    while let Ok((stream, _)) = listener.accept().await 
    {
        let mut games_lock = games.lock().await; // Lock the game hashmap
        player_queue.push(stream);//add players into queue

        if player_queue.len() >= 2 { // for make game with two player
            let game_id = games_lock.len() + 1; //give unique id to game
            let game_state = Arc::new(Mutex::new(GameState::new()));
            games_lock.insert(game_id, (*game_state.lock().await).clone()); // store game state and id in hashmap

            let player1 = player_queue.remove(0);
            let player2 = player_queue.remove(0);

            // Spawn tasks for two players
            tokio::spawn(handle_player(player1, game_state.clone(), 1, game_id));
            tokio::spawn(handle_player(player2, game_state.clone(), 2, game_id));

            println!("Game {} started with two players!", game_id);
        }
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

        // println!("Updated Board:\n{}", self.display_board());
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

async fn handle_player(stream: TcpStream, state: Arc<Mutex<GameState>>, player_id: usize,game_id: usize) 
{
    // split the stream into reader and writer
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = tokio::io::BufReader::new(reader);

    // give symbols to players
    let symbol = if player_id == 1 {"⭕"} else {"❌"};

    writer.write_all(format!("You are Player {} ({}) in Game {}\n", player_id, symbol, game_id).as_bytes()).await.unwrap();

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
        let position: usize = input.trim().parse().unwrap_or(usize::MAX);

        if position != usize::MAX && state.make_move(position, symbol).is_ok() 
        {
            println!("Updated Board: Game {}\n{}", game_id, state.display_board());
            state.current_player = if player_id == 1 { 2 } else { 1 };
            if let Some(winner) = state.winner_chacking() 
            {
                let message = if winner == "Draw" { "Game Over! It's a Draw 🤝".to_string() } 
                else { format!("Game Over! Winner: {}", winner) };

                writer.write_all(format!("Final Board:\n{}\n{}\nGame Finish\n", state.display_board(), message).as_bytes()).await.unwrap();
                state.game_over = true;

                writer.write_all("🤔 Do you want to play again? (yes/no):\n".as_bytes()).await.unwrap();
                let mut response = String::new();
                reader.read_line(&mut response).await.unwrap();
                if response.trim().to_lowercase() == "yes" 
                {
                    *state = GameState::new();
                    println!("Restart 🔄️!! Game {}", game_id);
                } else 
                {
                    writer.write_all("Bye!!🙋🙋 Thank you for playing!\n".as_bytes()).await.unwrap();
                    break;
                }
            }
        } else {
            writer.write_all("Invalid input. Try again.\n".as_bytes()).await.unwrap();
        }
        }
    }


