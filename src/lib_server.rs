//! Library for the game server

pub use super::*;
pub use std::io::{ stdin, Read, Write };
pub use std::net::{ TcpListener, TcpStream, Shutdown };
pub use std::str::from_utf8;
pub use std::sync::{ Arc, Mutex };
use std::string::FromUtf8Error;

const BUFFER_SIZE: usize = 50;
const MAX_N_BUFFERS: usize = 255;
const N_MILLISECONDS_WAIT: u64 = 10;
const N_MILLISECONDS_LONG_WAIT: u64 = 1000;
const YES_VALUES: [&str;10] = ["y", "yes", "yeah", "aye", "oui", "ja", "da", "ok", "si", "sim"];
const NO_VALUES: [&str;8] = ["n", "no", "nah", "nay", "non", "nein", "niet", "nope"];

/// check if a string is a synonym of ‘yes’
///
/// # Example
///
/// ```
/// use machiavelli::lib_server::is_yes;
///
/// let example_yes = "ja";
/// let example_no = "nein";
///
/// assert!(is_yes(example_yes));
/// assert!(!is_yes(example_no));
/// ```
pub fn is_yes(s: &str) -> bool {
    let s_l = s.to_lowercase();
    for &synonym in &YES_VALUES {
        if s_l == synonym {
            return true;
        }
    }
    false
}

/// check if a string is a synonym of ‘no’
///
/// # Example
///
/// ```
/// use machiavelli::lib_server::is_no;
///
/// let example_yes = "ja";
/// let example_no = "nein";
///
/// assert!(!is_no(example_yes));
/// assert!(is_no(example_no));
/// ```
pub fn is_no(s: &str) -> bool {
    let s_l = s.to_lowercase();
    for &synonym in &NO_VALUES {
        if s_l == synonym {
            return true;
        }
    }
    false
}

/// get the player name
pub fn handle_client(mut stream: TcpStream) -> Result<(TcpStream, String, usize), StreamError> {
    let mut player_name: String = "".to_string();
    match get_str_from_client(&mut stream) {
        Ok(s) => {
            // great the player
            player_name = s.clone();
            let msg = format!("Hello {s}!\nWaiting for other players to join...");
            stream.write_all(&[1])?;
            send_str_to_client(&mut stream, &msg)?;
        },
        Err(_)=> {
            println!("An error occured while reading the stream; terminating connection with {}", 
                     stream.peer_addr()?);
            stream.shutdown(Shutdown::Both)?;
        }
    };
    Ok((stream, player_name, 0))
}

/// get the player name and check that it is in the list of players and not already taken
pub fn handle_client_load(mut stream: TcpStream, names: &[String], names_taken: Arc<Mutex<Vec<String>>>) 
    -> Result<(TcpStream, String, usize), StreamError> 
{
    let mut player_name: String;
    let position: usize;
    loop {
        match get_str_from_client(&mut stream) {
            Ok(s) => {
                player_name = s.clone();
                
                // check if the name is in the list
                match names.iter().position(|x| x == &player_name) {
                    Some(i) => {
                        // check if it is not already taken
                        let mut lock = names_taken.lock().unwrap();
                        match lock.iter().position(|x| x == &player_name) {
                            Some(_) => {
                                stream.write_all(&[0])?;
                                let msg = "Sorry, this name is already taken!\n".to_string();
                                send_str_to_client(&mut stream, &msg)?;
                            },
                            None => {
                                position = i;
                                stream.write_all(&[1])?;
                                let msg = format!("Hello {s}!\nWaiting for other players to join...");
                                send_str_to_client(&mut stream, &msg)?;
                                lock.push(player_name.clone());
                                break;
                            }
                        }
                    },
                    None => {
                        stream.write_all(&[0])?;
                        let msg = format!("Sorry, {s} is not in the list of players!\n");
                        send_str_to_client(&mut stream, &msg)?;
                    }
                }

            },
            Err(_)=> {
                println!("An error occured while reading the stream; terminating connection with {}", 
                         stream.peer_addr()?);
                stream.shutdown(Shutdown::Both)?;
            }
        };
    }
    Ok((stream, player_name, position))
}

/// wait for a player to reconnect
pub fn wait_for_reconnection(stream: &mut TcpStream, name: &str, port: usize) 
    -> Result<(), StreamError>
{

    // wait for a connection

    // set-up the tcp listener
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;

    // get connections and check the player is the right one
    for mut new_stream in listener.incoming().flatten() {
        println!("New connection: {}", new_stream.peer_addr()?);

        // get the name 
        if let Ok(s) = get_str_from_client(&mut new_stream) {
            if s == name {
                new_stream.write_all(&[1]).unwrap_or(());
                send_str_to_client(&mut new_stream, 
                        &reset_style_string()).unwrap_or(());
                *stream = new_stream;
                break;
            } else {
                new_stream.write_all(&[2]).unwrap_or(());
                send_str_to_client(&mut new_stream, 
                        "Sorry; you're not the player we're expecting\n").unwrap_or(());
                new_stream.write_all(&[5]).unwrap_or(());
            }
        }
    }
    Ok(())
} 

/// player turn
#[allow(clippy::too_many_arguments)]
pub fn start_player_turn(table: &mut Table, hands: &mut [Sequence], deck: &mut Sequence, 
                         custom_rule_jokers: bool, player_names: &[String], current_player: usize, 
                         n_players: usize, streams: &mut [TcpStream], port: usize, 
                         sort_mode: &mut u8, previous_messages: &[String])
    -> Result<String,StreamError> {
    
    // copy the initial hand
    let hand_start_round = hands[current_player].clone();

    // copy the initial table
    let table_start_round = table.clone();
    
    // cards taken from the table
    let mut cards_from_table = Sequence::new();
    
    // send the instructions
    send_message_to_client(&mut streams[current_player], 
                           &format!("\u{0007}\n{}", instructions_no_save(true,false)))?;

    // get and process the player choice
    let mut message: String;
    loop {
        match get_message_from_client(&mut streams[current_player]) {
            Ok(mes) => {
                if mes.is_empty() {
                } else {
                    match mes[0] {
                    
                        // value 'e': end the turn
                        101 => {
                            if cards_from_table.number_cards() != 0 {
                                message = "You can't end your turn until you've played all the cards you've taken from the table!\n"
                                          .to_string();
                                send_message_to_client(&mut streams[current_player], &message)?;
                            } else if custom_rule_jokers && hands[current_player].contains_joker() {
                                message = "Jokers must be played!\n".to_string();
                                send_message_to_client(&mut streams[current_player], &message)?;
                            } else if hands[current_player].contains(&hand_start_round) {
                                match pick_a_card(&mut hands[current_player], deck) {
                                    Ok(card) => message = format!(" (you picked a {}{})", &card, &reset_style_string()),
                                    Err(_) => message = "No more card to draw!\n".to_string()
                                };
                                match *sort_mode {
                                    1 => hands[current_player].sort_by_rank(),
                                    2 => hands[current_player].sort_by_suit(),
                                    _ => ()
                                }
                                return Ok(message);
                            } else {
                                break
                            }
                        },
                    
                        // value 'p': play a sequence
                        112 => {
                            match play_sequence_remote(&mut hands[current_player], &mut cards_from_table,
                                                       table, &mes[1..]) {
                                Ok(None) => {
                                    
                                    // print the situation for the current player
                                    print_situation_remote(table, hands, deck, player_names, current_player,
                                                           current_player, &mut streams[current_player],
                                                           true, &cards_from_table, 
                                                           !hands[current_player].contains(&hand_start_round),
                                                           cards_from_table.number_cards() > 0, 
                                                           &previous_messages[current_player])?;

                                    // print the new situation for the other players
                                    for i in 0..n_players {
                                        if i != current_player {
                                            print_situation_remote(table, hands, deck, player_names, 
                                                                   i, current_player, &mut streams[i],
                                                                   false, &cards_from_table, false, false, 
                                                                   &previous_messages[i])?;
                                        }
                                    }

                                    // if the player has no more card and there is no card on the
                                    // table, end the turn 
                                    if (hands[current_player].number_cards() == 0) 
                                        && (cards_from_table.number_cards() == 0) {
                                        break;
                                    }
                                },

                                Ok(Some(s)) => {
                                    print_situation_remote(table, hands, deck, player_names, current_player,
                                                           current_player, &mut streams[current_player],
                                                           true, &cards_from_table, 
                                                           !hands[current_player].contains(&hand_start_round),
                                                           cards_from_table.number_cards() > 0,
                                                           &previous_messages[current_player])?;
                                    send_message_to_client(&mut streams[current_player], &s)?;
                                },

                                Err(_) => send_message_to_client(&mut streams[current_player], "Communication error\n")?
                            };
                        },
                        
                        // value 't': take a sequence from the table
                        116 => {
                            match take_sequence_remote(table, &mut cards_from_table, &mes[1..], 
                                                       &mut streams[current_player]) {
                                Ok(()) => {

                                    // print the new situation for the current player
                                    print_situation_remote(table, hands, deck, player_names, 
                                                           current_player, current_player, 
                                                           &mut streams[current_player], true, &cards_from_table,
                                                           false, cards_from_table.number_cards() > 0,
                                                           &previous_messages[current_player])?;

                                    // print the new situation for the other players
                                    for i in 0..n_players {
                                        if i != current_player {
                                            print_situation_remote(table, hands, deck, player_names, 
                                                                   i, current_player, &mut streams[i],
                                                                   false, &cards_from_table, false, false,
                                                                   &previous_messages[i])?;
                                        }
                                    }
                                },

                                Err(_) => send_message_to_client(&mut streams[current_player], "Communication error\n")?
                            };
                        },
                        
                        // value 'a': add cards to a sequence already on the table
                        97 => {
                            match add_to_table_sequence_remote(table, &mut hands[current_player], 
                                                               &mut cards_from_table, &mes[1..]) {
                                Ok(None) => {

                                    // print the new situation for the current player
                                    print_situation_remote(table, hands, deck, player_names, 
                                                           current_player, current_player, 
                                                           &mut streams[current_player], true, &cards_from_table,
                                                           !hands[current_player].contains(&hand_start_round),
                                                           cards_from_table.number_cards() > 0,
                                                           &previous_messages[current_player])?;

                                    // print the new situation for the other players
                                    for i in 0..n_players {
                                        if i != current_player {
                                            print_situation_remote(table, hands, deck, player_names, 
                                                                   i, current_player, &mut streams[i],
                                                                   false, &cards_from_table, false, false,
                                                                   &previous_messages[i])?;
                                        }
                                    }
                                    
                                    // if the player has no more card and there is no card on the
                                    // table, end the turn 
                                    if (hands[current_player].number_cards() == 0) 
                                        && (cards_from_table.number_cards() == 0) {
                                        break;
                                    }
                                },
                                Ok(Some(s)) => {
                                    print_situation_remote(table, hands, deck, player_names, 
                                                           current_player, current_player, 
                                                           &mut streams[current_player], true, &cards_from_table,
                                                           !hands[current_player].contains(&hand_start_round),
                                                           cards_from_table.number_cards() > 0, 
                                                           &previous_messages[current_player])?;
                                    send_message_to_client(&mut streams[current_player], &s)?;
                                },
                                Err(_) => send_message_to_client(&mut streams[current_player], "Communication error\n")?
                            };
                        },
 
                        // value 'r': sort cards by rank
                        114 => {
                            hands[current_player].sort_by_rank();
                            cards_from_table.sort_by_rank();
                            *sort_mode = 1;
                            print_situation_remote(table, hands, deck, player_names, current_player,
                                                   current_player, &mut streams[current_player],
                                                   true, &cards_from_table,
                                                   !hands[current_player].contains(&hand_start_round),
                                                   cards_from_table.number_cards() > 0, 
                                                   &previous_messages[current_player])?;
                        },
                        
                        // value 's': sort cards by suit
                        115 => {
                            hands[current_player].sort_by_suit();
                            cards_from_table.sort_by_suit();
                            *sort_mode = 2;
                            print_situation_remote(table, hands, deck, player_names, current_player,
                                                   current_player, &mut streams[current_player],
                                                   true, &cards_from_table, 
                                                   !hands[current_player].contains(&hand_start_round),
                                                   cards_from_table.number_cards() > 0,
                                                   &previous_messages[current_player])?;
                        },
            
                        // value 'g': give up on that round and take the penalty
                        103 => {
                            send_message_all_players(
                                streams,
                                &format!("{} resets the table and takes the penalty\n", 
                                         &player_names[current_player])
                            );
                            match cards_from_table.number_cards() {
                                0 => (),
                                _ => {
                                    give_up(table, &mut hands[current_player], deck, &hand_start_round, 
                                            &table_start_round, &mut cards_from_table);
                                    print_situation_remote(table, hands, deck, player_names, current_player,
                                                           current_player, &mut streams[current_player],
                                                           true, &cards_from_table, false, false,
                                                           &previous_messages[current_player])?;
                                }
                            }
                        },

                        _ => send_message_to_client(&mut streams[current_player], "Invalid input; please try again.")?,
                    }
                }
            },
            Err(_) => {
                send_message_all_players(
                    streams,
                    &format!("{} seems to have disconnected... Waiting for them to reconnect.\n", 
                             &player_names[current_player])
                );
                println!("Lost connection with player {}", current_player + 1);
                wait_for_reconnection(&mut streams[current_player], &player_names[current_player], port)?;
                println!("Player {} is back", current_player + 1);
                print_situation_remote(table, hands, deck, player_names, current_player,
                                       current_player, &mut streams[current_player],
                                       true, &cards_from_table, 
                                       !hands[current_player].contains(&hand_start_round),
                                       cards_from_table.number_cards() > 0,
                                       &previous_messages[current_player])?;
                send_message_all_players(
                    streams,
                    &format!("{} is back!\n", 
                             &player_names[current_player])
                );
            }
        };
    }
    Ok("".to_string())
}

fn play_sequence_remote(hand: &mut Sequence, cards_from_table: &mut Sequence,
                        table: &mut Table, mes: &[u8]) 
    -> Result<Option<String>, StreamError>
{
    // copy the initial hand and cards from tables
    let hand_copy = hand.clone();
    let cards_from_table_copy = cards_from_table.clone();

    // combine the hand and cards from the table
    let mut full_hand = hand.clone();
    let buffer = cards_from_table.clone();
    full_hand.merge(buffer.reverse());
  
    let mut seq = Sequence::new();
    
    let s = String::from_utf8(mes.to_vec())?;
    
    let mut seq_i_hand = Vec::<usize>::new();
    let mut seq_i_cft = Vec::<usize>::new();
    let n_hand = hand.number_cards();
    for item in s.trim().split(' ') {
        if let Ok(n) = item.parse::<usize>() {
            if n <= n_hand {
                let mut n_i = 0;
                for &i in &seq_i_hand {
                    if i < n {
                        n_i += 1;
                    }
                }
                let card = match hand.take_card(n-n_i) {
                    Some(c) => c,
                    None => continue
                };
                seq.add_card(card);
                seq_i_hand.push(n);
            } else {
                let m = n - n_hand;
                let mut n_i = 0;
                for &i in &seq_i_cft {
                    if i < m {
                        n_i += 1;
                    }
                }
                let card = match cards_from_table.take_card(m-n_i) {
                    Some(c) => c,
                    None => continue
                };
                seq.add_card(card);
                seq_i_cft.push(m);
            }
        }
    }

    if seq.is_valid() {
        table.add(seq);
        Ok(None)
    } else {
        *hand = hand_copy;
        *cards_from_table = cards_from_table_copy;
        let message = format!("{}{} is not a valid sequence!\n", 
                              &seq, &reset_style_string());
        Ok(Some(message))
    }
}

fn take_sequence_remote(table: &mut Table, hand: &mut Sequence, mes: &[u8], stream: &mut TcpStream) 
    -> Result<(), StreamError> 
{
    let content = String::from_utf8(mes.to_vec())?;
    let content = content.trim().split(' ');
    let mut seq_i = Vec::<usize>::new();
    for s in content {
        match s.parse::<usize>() {
            Ok(n) => {
                let mut n_i: usize = 0;
                for &i in &seq_i {
                    if i < n {
                        n_i += 1;
                    }
                }
                seq_i.push(n);
                match table.take(n-n_i) {
                    Some(seq) => {
                        hand.merge(seq.reverse());
                    },
                    None => send_message_to_client(stream, "This sequence is not on the table\n")?
                }
            },
            Err(_) => send_message_to_client(stream, "Error parsing the input!\n")?
        };
    }
    Ok(())
}

fn add_to_table_sequence_remote(table: &mut Table, hand: &mut Sequence, 
                                cards_from_table: &mut Sequence, mes: &[u8]) 
    -> Result<Option<String>, StreamError> 
{
    
    // copy the initial hand and cards from tables
    let hand_copy = hand.clone();
    let cards_from_table_copy = cards_from_table.clone();

    let mut seq_from_table: Sequence;
    let mut seq_from_hand = Sequence::new();
    let mut seq_from_hand_from_table = Sequence::new();

    // parse the request
    let content = String::from_utf8(mes.to_vec())?;
    let mut content = content.trim().split(' ');

    // parse the index of the sequence to which to add cards
    match content.next() {
        Some(x) => match x.parse::<usize>() {
            Ok(n) => match table.take(n) {
                Some(seq) => {
                    seq_from_table = seq;
                },
                None => {
                    let message = format!("Sequence {n} is not on the table\n");
                    return Ok(Some(message))
                }
            },
            Err(_) => {
                let message = "Error parsing the input!\n".to_string();
                return Ok(Some(message))
            }
        },
        None => return Ok(None)
    }

    // parse the sequence to play
    let mut seq_i_hand = Vec::<usize>::new();
    let mut seq_i_cft = Vec::<usize>::new();
    let n_hand = hand.number_cards();
    for s in content {
        if let Ok(n) = s.parse::<usize>() {
            if n <= n_hand {
                let mut n_i = 0;
                for &i in &seq_i_hand {
                    if i < n {
                        n_i += 1;
                    }
                }
                let card = match hand.take_card(n-n_i) {
                    Some(c) => c,
                    None => continue
                };
                seq_from_hand.add_card(card);
                seq_i_hand.push(n);
            } else {
                let m = n - n_hand;
                let mut n_i = 0;
                for &i in &seq_i_cft {
                    if i < m {
                        n_i += 1;
                    }
                }
                let card = match cards_from_table.take_card(m-n_i) {
                    Some(c) => c,
                    None => continue
                };
                seq_from_hand_from_table.add_card(card);
                seq_i_cft.push(m);
            }
        }
    }

    // clone the sequence from the table 
    let seq_from_table_org = seq_from_table.clone();

    // merge the sequences
    seq_from_hand.merge(seq_from_hand_from_table);
    seq_from_table.merge(seq_from_hand);

    // if it is valid, add it to the table; if not, restore the original situation
    if seq_from_table.is_valid() {
         table.add(seq_from_table);
         Ok(None)
    } else {
        *hand = hand_copy;
        *cards_from_table = cards_from_table_copy;
        table.add(seq_from_table_org);
        let message = format!("{}{} is not a valid sequence!\n", 
                              &seq_from_table, &reset_style_string());
        Ok(Some(message))
    }
}

#[allow(clippy::too_many_arguments)]
fn print_situation_remote(table: &Table, hands: &[Sequence], deck: &Sequence, 
                          player_names: &[String], player: usize, current_player: usize, 
                          stream: &mut TcpStream, print_instructions: bool, cards_from_table: &Sequence, 
                          has_played_something: bool, print_reset_option: bool, message: &str) 
    -> Result<(), StreamError>
{
    // string with the number of cards each player has
    let mut string_n_cards = format!("\nNumber of cards ({} remaining in the deck):", deck.number_cards());
    for i in 0..(hands.len()) {
        string_n_cards += &format!("\n  {}: {}", &player_names[i], &hands[i].number_cards());
    }
    string_n_cards += "\n";

    clear_and_send_message_to_client(stream, 
        &format!("\x1b[1m{}'s turn:{}", player_names[current_player], &reset_style_string()))?;
    send_message_to_client(stream, &string_n_cards)?;
    send_message_to_client(stream, &situation_to_string(table, &hands[player], cards_from_table, message))?;
    if print_instructions {
        send_message_to_client(stream, "\n")?;
        send_message_to_client(stream, &instructions_no_save(!has_played_something, print_reset_option))?;
    }
    Ok(())
}

/// send a message as a string to a client
pub fn send_str_to_client(stream: &mut TcpStream, s: &str) -> Result<(), StreamError> {
    send_bytes_to_client(stream, s.as_bytes())?;
    Ok(())
}

fn send_bytes_to_client_no_wait(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), StreamError> {
    
    // ensure that the number of bytes is small enough
    if bytes.len() > MAX_N_BUFFERS * BUFFER_SIZE {
        return Err(StreamError { message: format!(
                    "Stream too long: size: {}, maximum size: {}",
                    bytes.len(), MAX_N_BUFFERS*BUFFER_SIZE
                   ) })
    }

    // the first bytes will determine the number of times the buffer should be read
    let mut n_buffers: u8 = (bytes.len() / BUFFER_SIZE) as u8;
    if bytes.len() % BUFFER_SIZE != 0 {
        n_buffers += 1;
    }
    stream.write_all(&[n_buffers])?;

    // write the data stream
    for i in 0..((n_buffers-1) as usize) {
        stream.write_all(&bytes[i*BUFFER_SIZE..(i+1)*BUFFER_SIZE])?;
    }
    stream.write_all(&bytes[((n_buffers-1) as usize)*BUFFER_SIZE..])?;
    
    Ok(())
}

/// send a message as bytes to a client
pub fn send_bytes_to_client(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), StreamError> {
    
    send_bytes_to_client_no_wait(stream, bytes)?;
    
    // wait for a reply to be sent from the receiver
    stream.read_exact(&mut [0])?;
    
    Ok(())
}

/// get a message (string) from a client
pub fn get_str_from_client(stream: &mut TcpStream) -> Result<String, StreamError> {
    let bytes = get_bytes_from_client(stream)?;
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(_) => Err(StreamError::from(BytesToStringError {}))
    }
}

/// get a message (bytes) from a client
pub fn get_bytes_from_client(stream: &mut TcpStream) -> Result<Vec<u8>, StreamError> {
    
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

/// wait a moment
pub fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(N_MILLISECONDS_WAIT));
}

/// wait a longer moment
pub fn long_wait() {
    std::thread::sleep(std::time::Duration::from_millis(N_MILLISECONDS_LONG_WAIT));
}

/// check that no players have the same name; if yes, rename players
pub fn ensure_names_are_different(player_names: &mut [String], client_streams: &mut [TcpStream]) 
    -> Result<(), StreamError>
{
    let mut cont = true;
    while cont {
        cont = false;
        for i in 0..player_names.len() {
            for j in (i+1)..player_names.len() {
                if player_names[j] == player_names[i] {
                    cont = true;
                    match String::from_utf8(send_message_get_reply(&mut client_streams[j], 
                                       &format!("The name {} is already taken! Please choose a different one.\n",
                                                &player_names[j]))?) {
                        Ok(n) => player_names[j] = n,
                        Err(_) => send_message_to_client(&mut client_streams[j], "Could not read the input!")?
                    }
                }
            }
        }
    }
    Ok(())
}

/// send the instruction to send a message to the client, and read the response as a string
pub fn get_string_from_client(stream: &mut TcpStream) -> Result<String, StreamError> {
    let msg = get_message_from_client(stream)?;
    match String::from_utf8(msg) {
        Ok(s) => Ok(s),
        Err(_) => Err(StreamError { message: "Could not convert the input to a string".to_string() })
    }
}

fn get_message_from_client(stream: &mut TcpStream) -> Result<Vec<u8>, StreamError>{
    stream.write_all(&[4])?;
    get_bytes_from_client(stream)
}

/// send the instruction to clear the screen and send back a message to the client, and read the 
/// response as a string
pub fn clear_and_send_message_to_client(stream: &mut TcpStream, msg: &str) -> Result<(), StreamError>{
    stream.write_all(&[2])?;
    send_str_to_client(stream, msg)
}

/// send the instruction to print a message to the client, then send a message to the same client
pub fn send_message_to_client(stream: &mut TcpStream, msg: &str) -> Result<(), StreamError>{
    stream.write_all(&[1])?;
    send_str_to_client(stream, msg)
}

/// send a message and get the response
pub fn send_message_get_reply(stream: &mut TcpStream, message: &str) 
    -> Result<Vec<u8>, StreamError>
{
    stream.write_all(&[3])?;
    send_str_to_client(stream, message)?;
    get_bytes_from_client(stream)
}

/// send the same message to all players
pub fn send_message_all_players(client_streams: &mut [TcpStream], message: &str) {

    // send the messages
    for cs in client_streams.iter_mut() {
        cs.write_all(&[1]).unwrap_or(());
        send_bytes_to_client_no_wait(cs, message.as_bytes()).unwrap_or(());
    }

    // wait until all clients have confirmed reception
    for cs in client_streams.iter_mut() {
        cs.read_exact(&mut [0]).unwrap_or(());
    }
    
}

/// clear the screens and send the same message to all players
pub fn clear_and_send_message_all_players(client_streams: &mut [TcpStream], message: &str) {

    // send the messages
    for cs in client_streams.iter_mut() {
        cs.write_all(&[2]).unwrap_or(());
        send_bytes_to_client_no_wait(cs, message.as_bytes()).unwrap_or(());
    }

    // wait until all clients have confirmed reception
    for cs in client_streams.iter_mut() {
        cs.read_exact(&mut [0]).unwrap_or(());
    }
    
}

// errors

#[derive(Debug)]
pub struct StreamError {
    message: String
}

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

impl std::convert::From<FromUtf8Error> for StreamError {
    fn from(error: FromUtf8Error) -> Self {
        StreamError { message: format!("UTF-8 error: {}", &error) }
    }
}
