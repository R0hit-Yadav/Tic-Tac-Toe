# Multi-Player Networked Tic-Tac-Toe Game

A simple multiplayer Networked Tic-Tac-Toe game server built with Rust and `tokio`. 

The server listens for client connections, pairs two players, and facilitates the game where players take turns to mark their respective positions on the grid. 

The game continues until there's a winner or a tie, and players can choose to play again.

## Features

- **Multiplayer Support**: Two players are matched together to play the game.
- **Real-time Gameplay**: Players can see each other's moves and the updated game board in real-time.
- **Turn-based System**: Players take turns to make their move.
- **Game Over Conditions**: The game ends when a player wins or when there is a tie.
- **Replay Option**: Players can choose to play again after a game ends.
- **Client Handling**: The server handles multiple clients and matches them automatically when two players are available.

## Prerequisites

- Rust installed on your machine. You can download it from [here](https://www.rust-lang.org/learn/get-started).
- `tokio` for async runtime (the server is built using the `tokio` async framework).

## Getting Started

1. **Clone the repository**:

    ```bash
    git clone https://github.com/R0hit-Yadav/Tic-Tac-Toe.git
    cd Tic-Tac-Toe
    ```

2. **Build the server**:

    Use `cargo` to build the project:

    ```bash
    cargo build --release
    ```

3. **Run the server**:

    Start the server:

    ```bash
    cargo run
    ```

    The server will start listening on `127.0.0.1:8080`.

4. **Connect with clients**:

    - The server uses TCP to communicate with clients. To connect, you can either create a separate client that connects to `127.0.0.1:8080`, or use a tool like `telnet` or `nc` to test the server:
  
    ```bash
    telnet 127.0.0.1 8080
    ```

    Two clients can connect to the server, and they will be paired together automatically to start a game.

## How It Works

1. **Server Initialization**: The server listens for incoming TCP connections on `127.0.0.1:8080`.
2. **Client Connection**: When a client connects, they are added to a queue of waiting clients.
3. **Pairing Players**: When at least two players are in the queue, the server pairs them and starts a game session.
4. **Gameplay**:
    - Players take turns to enter a number between 1-9, which corresponds to a position on the Tic-Tac-Toe grid.
    - The server sends the updated grid to both players after every move.
    - The game continues until one player wins or the board is filled, resulting in a tie.
5. **Replay Option**: After each game, players can choose to play again. If they agree, the game restarts; otherwise, they are removed from the game session.

## Game Rules

- The board is a 3x3 grid, with positions numbered from 1 to 9:
1 | 2 | 3
4 | 5 | 6
7 | 8 | 9

- Player 1 uses `❌` (X), and Player 2 uses `⭕` (O).
- Players take turns entering their moves by typing a number corresponding to an empty cell on the board.
- The first player to get three of their symbols in a row (vertically, horizontally, or diagonally) wins.
- If all the cells are filled and no player has won, the game ends in a tie.

## Game Board Example

Here is an example of the game board during play:
❌ | 2 | 3
4  | ⭕| 6
7  | 8  | 9


## Client Protocol

- **Player 1 (❌)**: When it’s Player 1’s turn, the prompt will ask them to enter a number (1-9) to make their move.
- **Player 2 (⭕)**: The same applies for Player 2 when it’s their turn.
- If a player tries to move out of turn, they will receive a "Not your turn" message.
- If a player disconnects, the other player wins by default.

## Notes

- The game uses `tokio` and `async/await` for handling multiple clients concurrently.
- The server is designed to pair players as soon as two clients are available. If a player disconnects, they are removed from the queue, and the remaining player will have to wait for a new opponent.
- This project could be extended by adding more features like a scoring system, or supporting more than two players in future iterations.

Thanks For Watching 
  
