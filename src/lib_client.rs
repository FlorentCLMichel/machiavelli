//! Library for the game client

use super::*;
pub use std::net::TcpStream;
pub use std::io::{ Read, Write };
pub use std::str::from_utf8;

const BUFFER_SIZE: usize = 50;
const MAX_N_BUFFERS: usize = 255;
const N_MILLISECONDS_WAIT: u64 = 10;

// ask for the port
fn get_address() -> String {
    println!("Address and port of the server?");
    loop {
        match get_input() {
            Ok(s) => return s.trim().to_string(),
            Err(_) => println!("Could not parse the input")
        };
    }
}

/// try to connect to the server and send the player name
///
/// If the connection is successful, clear the terminal, print the reply from the server, and
/// return a `TcpStream`. 
/// If not, return a `StreamError`.
pub fn say_hello(mut name: String) -> Result<TcpStream, StreamError> {

    // host address
    let name_file_port_server = "Config/port_client.dat";
    let host = match std::fs::read_to_string(name_file_port_server) {
        Ok(s) => s.trim().to_string(),
        Err(_) => get_address()
    };

    match TcpStream::connect(&host) {
        Ok(mut stream) => {
            println!("Successfully connected to {}", &host);
            
            loop {
                
                if name.is_empty() {
                    // get the player name
                    let mut cont = true;
                    println!("Player name:");
                    while cont {
                        match get_input() {
                            Ok(s) => {
                                name = s.trim().to_string();
                                cont = false
                            },
                            Err(_) => println!("Could not parse the input")
                        };
                    }
                }

                send_str_to_server(&mut stream, &name)?;
                println!("Sent the name to server; awaiting reply...");
    
                let mut buffer: [u8; 1] = [0];
                stream.read_exact(&mut buffer)?;
                match buffer[0] {
                    1 => {
                        match get_str_from_server(&mut stream) {
                            Ok(s) => {
                                
                                // set the terminal appearance
                                reset_style();

                                // clear the terminal
                                clear_terminal();

                                // print the message sent by the server
                                println!("{s}");
                            }
                            Err(e) => {
                                println!("Failed to receive data: {e}");
                            }
                        }
                        break;
                    },
                    2 => {
                        match get_str_from_server(&mut stream) {
                            Ok(s) => { 
                                // print the message sent by the server
                                println!("{s}");
                            }
                            Err(e) => {
                                println!("Failed to receive data: {e}");
                            }
                        }
                        break;
                    },
                    _ => {
                        name.clear();
                        println!("{}", get_str_from_server(&mut stream)?)
                    }
                };
            }
            Ok(stream)
        }
        Err(e) => { Err(StreamError::from(e)) }
    }
}

/// get a request from the server and act accordingly
///
/// The request is initially encoded in a single byte sent by the server to `stream`. 
/// Five values are currently supported: 
///
/// * 1: print the next message sent by the server
/// * 2: clear the terminal and print the next message sent by the server
/// * 3: print the next message sent by the server and send back a message from stdin
/// * 4: send a message from stdin
/// * 5: close the client
pub fn handle_server_request(single_byte_buffer: &mut [u8; 1], stream: &mut TcpStream) -> Result<(), StreamError> {
    stream.read_exact(single_byte_buffer)?;
    match single_byte_buffer[0] {
        
        // value 1: print the message from the server
        1 => print_str_from_server(stream)?,
        
        // value 2: clear the terminal and print the message from the server
        2 => clear_and_print_str_from_server(stream)?,
        
        // value 3: print the message and return a reply in bytes
        3 => print_and_reply(stream)?,
        
        // value 4: send a message
        4 => send_message(stream)?,
        
        // value 5: exit
        5 => {
            print!("\x1b[0m\x1b[?25h"); // reset the style and show the cursor
            print!("\x1b[2J\x1b[1;1H"); // clear the screen
            print!("\x1b[K"); // redraw the screen
            std::process::exit(0)
        },

        _ => ()
    };
    Ok(())
}

fn clear_and_print_str_from_server(stream:  &mut TcpStream) -> Result<(), StreamError> {
    clear_terminal();
    println!("{}", get_str_from_server(stream)?);
    Ok(())
}

fn print_str_from_server(stream:  &mut TcpStream) -> Result<(), StreamError> {
    print!("{}", get_str_from_server(stream)?);
    Ok(())
}

fn print_and_reply(stream:  &mut TcpStream) -> Result<(), StreamError> {
    println!("{}", get_str_from_server(stream)?);
    send_message(stream)
}

fn send_message(stream:  &mut TcpStream) -> Result<(), StreamError> {
    let mut reply = String::new();
    let mut cont = true;
    while cont {
        match get_input() {
            Ok(s) => {
                reply = s.trim().to_string();
                cont = false
            },
            Err(_) => println!("Could not parse the input")
        };
    }
    send_str_to_server(stream, &reply)?;
    Ok(())
}

/// convert a string to a sequence of bytes and send it to the server
pub fn send_str_to_server(stream: &mut TcpStream, s: &str) -> Result<(), StreamError> {
    send_bytes_to_server(stream, s.as_bytes())?;
    Ok(())
}

/// send a sequence of bytes to the server and wait for confirmation that it has been received
pub fn send_bytes_to_server(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), StreamError> {
    
    // ensure that the number of bytes is small enough
    if bytes.len() > MAX_N_BUFFERS * BUFFER_SIZE {
        return Err(StreamError { message: format!(
                    "Stream too long: size: {}, maximum size: {}",
                    bytes.len(), MAX_N_BUFFERS*BUFFER_SIZE
                   ) })
    }

    // the first bytes will determine the number of times the buffer should be read
    let mut n_buffers: u8 = (bytes.len() / BUFFER_SIZE) as u8;
    if !bytes.len().is_multiple_of(BUFFER_SIZE) {
        n_buffers += 1;
    }
    stream.write_all(&[n_buffers])?;

    // write the data stream
    for i in 1..(n_buffers as usize) {
        stream.write_all(&bytes[(i-1)*BUFFER_SIZE..i*BUFFER_SIZE])?;
    }
    if n_buffers > 0 {
        stream.write_all(&bytes[((n_buffers-1) as usize)*BUFFER_SIZE..])?;
    }

    // wait for a reply to be sent from the receiver
    while stream.read_exact(&mut [0]).is_err() {}
    
    Ok(())
}

/// get a sequence of bytes from the server and convert it to a string
pub fn get_str_from_server(stream: &mut TcpStream) -> Result<String, StreamError> {
    let bytes = get_bytes_from_server(stream)?;
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(_) => Err(StreamError::from(BytesToStringError {}))
    }
}

/// get a sequence of bytes from the server
pub fn get_bytes_from_server(stream: &mut TcpStream) -> Result<Vec<u8>, StreamError> {
    
    // buffer
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    // the first bytes will determine the number of times the buffer should be read
    let mut n_buffers: [u8; 1] = [0];
    stream.read_exact(&mut n_buffers)?;

    // vector containing the result
    let mut res = Vec::<u8>::new();

    // read the data stream
    let mut size;
    for _i in 0..n_buffers[0] {
        size = stream.read(&mut buffer)?;
        res.extend_from_slice(&buffer[..size]);
    }
   
    // send something to confirm I have received the data
    stream.write_all(&[0])?;

    // return the result
    Ok(res)
}

/// wait a moment (`N_MILLISECONDS_WAIT` in milliseconds)
pub fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(N_MILLISECONDS_WAIT));
}


// errors

/// generic error raised when reading from or writing to a stream fails
#[derive(Debug)]
pub struct StreamError {
    message: String
}

/// generic error raised when conversion from a sequence of bytes to a string fails
#[derive(Debug)]
pub struct BytesToStringError {}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "StreamError: {}", self.message)
    }
}

impl std::convert::From<std::io::Error> for StreamError {
    fn from(error: std::io::Error) -> Self {
        StreamError { message: format!("IO Error: {error}") }
    }
}

impl std::convert::From<BytesToStringError> for StreamError {
    fn from(_error: BytesToStringError) -> Self {
        StreamError { message: "Could not convert the byte sequence to a string".to_string() }
    }
}
