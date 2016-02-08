extern crate ws;
extern crate env_logger;
extern crate serde_json;
extern crate clap;
extern crate ansi_term;

use ws::{connect, CloseCode};
use serde_json::Value;
use clap::{Arg, App};
use ansi_term::Colour::Fixed;

fn main() {
    let matches =
	App::new("gs-firehose")
		.version("0.1")
		.about("Connects to a grab-site or ArchiveBot server and dumps all messages in either a human-readable or JSON format.")
		.arg(Arg::with_name("WS_URL")
			.help("The WebSocket URL to connect to.  Default: ws://127.0.0.1:29001")
			.index(1))
		.get_matches();

	let url = matches.value_of("WS_URL").unwrap_or("ws://127.0.0.1:29001");

	// Set up logging.  Set the RUST_LOG env variable to see output.
	env_logger::init().unwrap();

	let gray = Fixed(244);

	if let Err(error) = connect(url, |out| {
		// Queue a message to be sent when the WebSocket is open
		if let Err(_) = out.send(r#"{"type": "hello", "mode": "dashboard"}"#) {
			println!("Websocket couldn't queue an initial message.")
		}

		// The handler needs to take ownership of out, so we use move
		move |msg: ws::Message| {
			let text = msg.as_text().unwrap();
			let ev: Value = serde_json::from_str(text).unwrap();
			//println!("{:?}", ev);
			let ident = ev.lookup("job_data.ident").unwrap().as_string().unwrap();
			if let Some(message) = ev.find("message") {
				let trimmed = message.as_string().unwrap().trim_right();
				if !trimmed.is_empty() {
					for line in trimmed.lines() {
						if line.starts_with("ERROR ") {
							println!("{} {}", gray.paint(ident), line);
						} else {
							println!("{}  {}", gray.paint(ident), line);
						}
					}
				}
			} else {
				let response_code = ev.find("response_code").unwrap().as_u64().unwrap();
				let url = ev.find("url").unwrap().as_string().unwrap();
				println!("{}   {:>3} {}", gray.paint(ident), response_code, url);
			}

			out.close(CloseCode::Normal)
		}

	}) {
		println!("Failed to create WebSocket due to: {:?}", error);
	}
}
