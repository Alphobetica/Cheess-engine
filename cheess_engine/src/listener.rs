use std::sync::{Mutex, Arc};
use crate::InputType;
use crate::UserInput;
use crate::InputType::*;

type Input = Arc<Mutex<UserInput>>;
pub fn listen(user_input: Input) -> std::io::Result<()> {
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
		conn.get_mut().write_all(b"Server response\n")?;

		// Print out the result, getting the newline for free!
		print!("Client answered: {buffer}");

		// Clear the buffer so that the next iteration will display new data instead of messages
		// stacking on top of one another, like a queue but instead its just a queue
		match buffer.to_lowercase().as_str().trim() {
			"resign" => {
				user_input.lock().unwrap().input_queue.push_back(Resign);
			},
      "exit" => {
				user_input.lock().unwrap().input_queue.push_back(Exit);
				return Ok(());
      },
      "reset" => {
        user_input.lock().unwrap().input_queue.push_back(Reset);
      },
			"\"default\"" => {
				user_input.lock().unwrap().input_queue.push_back(InputType::GameMode(crate::GameMode::Default));
			},
			"\"blitz\"" => {
				user_input.lock().unwrap().input_queue.push_back(InputType::GameMode(crate::GameMode::Blitz));
			},
			val => {
        //moove
				user_input.lock().unwrap().input_queue.push_back(Move(val.to_string()));
      },
		}

		buffer.clear();
	}
	Ok(())
}

