use std::sync::{Mutex, Arc};
use std::collections::VecDeque;
use crate::{ BitBoard, UserInput, InputType, GameState, boardrep_to_bitboard, MoveError};
use crate::InputType::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Response {
	pub board: BitBoard,
	pub timer_white: std::time::Duration,
	pub timer_black: std::time::Duration,
	pub player_turn: u8,
	pub game_end: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerResponse {
	Response(Response),
	Error(MoveError),

}

#[derive(Debug)]
pub struct ResponseQueue {
	pub res_queue: VecDeque<ServerResponse>
}

type Input = Arc<Mutex<UserInput>>;
type ResponsePtr = Arc<Mutex<ResponseQueue>>;

pub fn listen(user_input: Input, response: ResponsePtr) -> std::io::Result<()> {
	use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
	use std::io::{self, prelude::*, BufReader};

	// Define a function that checks for errors in incoming connections. We'll use this to filter
	// through connections that fail on initialization for one reason or another.
	fn handle_error(conn: io::Result<Stream>) -> Option<Stream> {
		match conn {
			Ok(c) => Some(c),
			Err(e) => {
				eprintln!("Incoming connection failed: {e}");
				None
			}
		}
	}

	// Pick a name.
	let printname = "cheess.sock";
	let name = printname.to_ns_name::<GenericNamespaced>()?;

	// Configure our listener...
	let opts = ListenerOptions::new().name(name);

	// ...then create it.
	let listener = match opts.create_sync() {
		Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
			eprintln!(
				"Error: could not start server because the socket file is occupied. Please check if
				{printname} is in use by another process and try again."
			);
			return Err(e);
		}
		x => x?,
	};

	// The syncronization between the server and client, if any is used, goes here.
	eprintln!("Server running at {printname}");

	// Preemptively allocate a sizeable buffer for receiving at a later moment. This size should
	// be enough and should be easy to find for the allocator. Since we only have one concurrent
	// client, there's no need to reallocate the buffer repeatedly.
	let mut buffer = String::with_capacity(128);

	for conn in listener.incoming().filter_map(handle_error) {
		// Wrap the connection into a buffered receiver right away
		// so that we could receive a single line from it.
		let mut conn = BufReader::new(conn);
		println!("Incoming connection!");

		// Since our client example sends first, the server should receive a line and only then
		// send a response. Otherwise, because receiving from and sending to a connection cannot
		// be simultaneous without threads or async, we can deadlock the two processes by having
		// both sides wait for the send buffer to be emptied by the other.
		conn.read_line(&mut buffer)?;

		// Now that the receive has come through and the client is waiting on the server's send, do
		// it. (`.get_mut()` is to get the sender, `BufReader` doesn't implement a pass-through
		// `Write`.)


		// Print out the result, getting the newline for free!
		
		// Clear the buffer so that the next iteration will display new data instead of messages
		// stacking on top of one another, like a queue but instead its just a queue
		let res = match buffer.to_lowercase().as_str().trim() {
			// TODO: fix this response stuff
			"resign" => {
				user_input.lock().unwrap().input_queue.push_back(Resign);
				// TOOD: Fix this, should not be an error
				ServerResponse::Error(MoveError::InvalidMove)
			},
      "exit" => {
				user_input.lock().unwrap().input_queue.push_back(Exit);
				return Ok(());
      },
      "reset" => {
        user_input.lock().unwrap().input_queue.push_back(Reset);
				// TOOD: Fix this, should not be an error
				ServerResponse::Error(MoveError::InvalidMove)
      },
			"\"default\"" => {
				user_input.lock().unwrap().input_queue.push_back(InputType::GameMode(crate::GameMode::Default));
				let game = GameState::new();
				ServerResponse::Response(Response { 
					board: boardrep_to_bitboard(&game.board),
					timer_black: game.black_timer,
					timer_white: game.white_timer,
					player_turn: game.player_turn,
					game_end: game.game_over,
				})

			},
			"\"blitz\"" => {
				user_input.lock().unwrap().input_queue.push_back(InputType::GameMode(crate::GameMode::Blitz));
				let mut game = GameState::new();
				game.blitz_mode();
				ServerResponse::Response(Response { 
					board: boardrep_to_bitboard(&game.board),
					timer_black: game.black_timer,
					timer_white: game.white_timer,
					player_turn: game.player_turn,
					game_end: game.game_over,
				})
			},
			val => {
        //moove
				user_input.lock().unwrap().input_queue.push_back(Move(val.to_string()));
				let res = 'move_loop: loop {
					std::thread::sleep(std::time::Duration::from_millis(50));
					let mut lock = response.lock().expect("Panic locking response queue from listener");
					if let Some(res) = lock.res_queue.pop_front() {
						break 'move_loop res;
					}
				};
				res
      },
		};
		
		// maybe (hopefully) panics
	  let res = serde_json::to_string(&res)?;
		// res.push('\n');
		conn.get_mut().write_all(res.as_bytes())?;
		// Write response here
		
		
		// conn.get_mut().write_all(b"Server response\n")?;
		
		
		print!("Client answered: {buffer}");
		buffer.clear();
	}		
	Ok(())
}

