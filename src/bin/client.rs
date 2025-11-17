//! Client for the Machiavelli game

use std::env;
use std::process::exit;
use machiavelli::lib_client::*;

fn main() {

    ctrlc::set_handler(|| {
        print!("\x1b[0m\x1b[?25h"); // reset the style and show the cursor
        print!("\x1b[2J\x1b[1;1H"); // clear the screen
        print!("\x1b[K"); // redraw the screen
        exit(0);
    }).expect("Could not set the Ctrl-C signal handler!");

    // parse the command-line arguments
    let args: Vec<String> = env::args().collect();

    let single_byte_buffer: &mut [u8; 1] = &mut [0];

    // set-up the TCP stream to communicate with the server
    let mut stream = if args.len() > 1 {
        
        // if one command-line argument is given, use it as player name
        connect(&args[1])

    } else {
        
        //otherwise, the name will be asked
        connect("")
    };

    loop {

        // handle the server request and quit if the server can not be reached
        handle_server_request(single_byte_buffer, &mut stream).unwrap_or_else(|_| {
            println!("lost connection to the server");
            print!("\x1b[0m\x1b[?25h"); // reset the style and show the cursor
            print!("\x1b[K"); // redraw the screen
            exit(1);
        });

    }
}


// function to try to connect to the server and exit if unsuccessful
fn connect(name: &str) -> TcpStream {
    match say_hello(name.to_string()) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect: {e}");
            exit(1);
        }
    }
}
