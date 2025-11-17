//! # Machiavelli
//!
//! A simple machiavelli card game *(work in progress)*


use std::io::{ stdin, Write };
pub mod sequence_cards;
pub mod table;
pub mod sort;
pub mod encode;
pub mod lib_server;
pub mod lib_client;
pub use sequence_cards::*;
pub use table::*;

/// number of cards to take when resetting 
pub const PENALTY_RESET: usize = 3;

pub fn reset_style_string() -> String {
    [
        "\x1b[0m", // reset attributes
        "\x1b[30;47m", // set the foreground and background colours
        "\x1b[?25l", // hide the cursor
        "\x1b[K" // redraw the prompt
    ].join("")
}

/// reset the terminal output style
pub fn reset_style() {
    print!("{}", reset_style_string());
}

/// clear the terminal
pub fn clear_terminal() {
    print!("\x1b[2J\x1b[1;1H");
}


/// Structure to store the game configuration
#[derive(Debug, PartialEq)]
pub struct Config {
    pub n_decks: u8,
    pub n_jokers: u8,
    pub n_cards_to_start: u16,
    pub custom_rule_jokers: bool,
    pub n_players: u8
}


impl Config {

    /// Convert the config structure to a sequence of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::Config;
    ///
    /// let config = Config {
    ///     n_decks: 2,
    ///     n_jokers: 4,
    ///     n_cards_to_start: 13,
    ///     custom_rule_jokers: false,
    ///     n_players: 2
    /// };
    ///
    /// let config_bytes = config.to_bytes();
    ///
    /// assert_eq!(
    ///     vec![2,4,0,13,0,2], 
    ///     config_bytes);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.n_decks,
            self.n_jokers,
            (self.n_cards_to_start >> 8) as u8,
            (self.n_cards_to_start & 255) as u8,
            self.custom_rule_jokers as u8,
            self.n_players
        ]
    }

    /// Get a config from a vector of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::Config;
    ///
    /// let bytes: Vec<u8> = vec![2,4,0,13,0,2];
    ///
    /// let config = Config::from_bytes(&bytes);
    ///
    /// let expected_config = Config {
    ///     n_decks: 2,
    ///     n_jokers: 4,
    ///     n_cards_to_start: 13,
    ///     custom_rule_jokers: false,
    ///     n_players: 2
    /// };
    ///
    /// assert_eq!(expected_config, config);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Config {
        Config {
            n_decks: bytes[0],
            n_jokers: bytes[1],
            n_cards_to_start: (bytes[2] as u16)*256 + (bytes[3] as u16),
            custom_rule_jokers: bytes[4] != 0,
            n_players: bytes[5]
        }
    }
}

/// get the vector of player names from a file
pub fn load_names(fname: &str) -> Result<Vec<String>, InvalidInputError> {
    let content = std::fs::read_to_string(fname)?;
    Ok(content.trim().split('\n').map(String::from).collect())
}

/// save the vector of player names to a file
pub fn save_names(names: &[String], fname: &str) -> Result<(), InvalidInputError> {
    let names_single_string = names.join("\n");
    let mut file = std::fs::File::create(fname)?;
    file.write_all(names_single_string.as_bytes())?;
    Ok(())
}

fn first_word(s: &str) -> Result<String,InvalidInputError> {
    match s.split(' ').next() {
        Some(res) => Ok(res.to_string()),
        None => Err(InvalidInputError {})
    }
}

/// load the config from a file
pub fn get_config_from_file(fname: &str) -> Result<(Config,String),InvalidInputError> {
    
    // open the file
    let content = std::fs::read_to_string(fname)?;
    let content: Vec<&str> = content.split('\n').collect();

    // check that the file has at least the right number of lines
    if content.len() < 6 {
        return Err(InvalidInputError {});
    }

    // get the config
    let n_decks = first_word(content[0])?.parse::<u8>()?;
    let n_jokers = first_word(content[1])?.parse::<u8>()?;
    let n_cards_to_start = first_word(content[2])?.parse::<u16>()?;
    let custom_rule_jokers = first_word(content[3])? == "1";
    let n_players = first_word(content[4])?.parse::<u8>()?;
    let savefile = first_word(content[5])?;
   
    // print the parameters
    #[allow(clippy::print_literal)] {
        println!("{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
                 "Number of decks",
                 n_decks,
                 "Number of jokers",
                 n_jokers,
                 "Number of starting cards",
                 n_cards_to_start,
                 "Jokers can't be kept",
                 custom_rule_jokers,
                 "Number of players",
                 n_players,
                 "Savefile", 
                 savefile);
    }

    Ok((Config {
        n_decks,
        n_jokers,
        n_cards_to_start,
        custom_rule_jokers,
        n_players
    }, savefile))
}

/// ask the user for the game information and savefile name
pub fn get_config_and_savefile() -> Result<(Config, String),InvalidInputError> {
    let conf = get_config()?;
    println!("Name of the save file: ");
    let savefile = get_input()?.trim().to_string();
    Ok((conf, savefile))
}

/// ask the user for the game information and return a Config
pub fn get_config() -> Result<Config,InvalidInputError> {
    
    println!("Number of decks (integer between 1 and 255) (enter 0 to load a previously saved game): ");
    let mut n_decks: u8 = 0;
    let mut load = false;
    while n_decks == 0 {
        n_decks = match get_input()?.trim().parse::<u8>() {
            Ok(0) => {
                load = true;
                1
            },
            Ok(n) => n,
            Err(_) => {
                println!("Invalid input");
                0
            }
        };
    }

    if load {
        return Ok(Config {
            n_decks: 0,
            n_jokers: 0,
            n_cards_to_start: 0,
            custom_rule_jokers: false,
            n_players: 0
        });
    }
    
    println!("Number of jokers (integer between 0 and 255): ");
    let mut n_jokers: u8 = 0; 
    let mut set = false;
    while !set {
        n_jokers = match get_input()?.trim().parse::<u8>() {
            Ok(n) => {
                set = true;
                n
            },
            Err(_) => {
                println!("Invalid input");
                0
            }
        };
    }
    
    println!("Number of cards to start with (integer): ");
    let mut n_cards_to_start: u16 = 0;
    while n_cards_to_start == 0 {
        n_cards_to_start = match get_input()?.trim().parse::<u16>() {
            Ok(n) => {
                let mut res = 0;
                if n==0 {
                    println!("You need to start with at least one card");
                } else if n > ((52 * (n_decks as u16)) + (n_jokers as u16)) {
                    println!("You can't draw more cards than there are in the deck");
                } else {
                    res = n;
                }
                res
            },
            Err(_) => return Err(InvalidInputError {})
        };
    }
    
    println!("Custom rule—jokers must be played immediately (y/n): ");
    let custom_rule_jokers = matches!(get_input()?.trim(), "y");
    
    println!("Number of players: ");
    let mut n_players = 0;
    while n_players == 0 {
        n_players = match get_input()?.trim().parse::<u8>() {
            Ok(0) => {
                println!("I need at least one player!");
                0
            }
            Ok(n) => n,
            Err(_) => {
                println!("Could not parse the input");
                0
            }
        };
    }

    Ok(Config {
        n_decks, 
        n_jokers,
        n_cards_to_start,
        custom_rule_jokers,
        n_players
    })
}

fn instructions() -> String {
    format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        "q: Save and quit",
        "c: Pick a card",
        "p: Play a sequence",
        "t: Take from the table",
        "a: Pass",
        "r, s: Sort cards by rank or suit",
        "g: Give up and reset"
        )
}

pub fn instructions_no_save(must_pick_a_card: bool, print_reset_option: bool) 
    -> String 
{
    let mut will_pick_a_card = &"";
    let mut reset_option = &"";
    if must_pick_a_card {
        will_pick_a_card = &" (and pick a card)";
    }
    if print_reset_option {
        reset_option = &"g: Give up and reset\n";
    }
    format!("{}{}\n{}\n{}\n{}\n{}\n{}\n",
        "e: End your turn",
        will_pick_a_card,
        "p x y ...: Play the sequence x y ...",
        "t x y ...: Take the sequences x, y, ... from the table",
        "a x y z ...: Add the sequence y z ... to sequence x on the table",
        "r, s: Sort cards by rank or suit",
        reset_option
        )
}

pub fn player_turn(table: &mut Table, hand: &mut Sequence, deck: &mut Sequence, 
                   custom_rule_jokers: bool, player_name: &str) -> bool {

    // copy the initial hand
    let hand_start_round = hand.clone();
    
    // copy the initial table
    let table_start_round = table.clone();

    // get the player choice
    let mut message = String::new();
    loop {
        
        // clear the terminal
        clear_terminal();
        
        println!("\x1b[1m{player_name}'s turn");
        reset_style();
        
        print_situation(table, hand, deck);

        // print the options
        println!("{}", &instructions());
        
        if message.is_empty() {
            println!("\n{message}");
            message.clear()
        }
        
        match get_input().unwrap_or_else(|_| {"".to_string()}).trim() {
            "q" => {
                if !hand_start_round.contains(hand) {
                    message = "You can't save until you've played all the cards you've taken from the table!".to_string();
                } else if !hand.contains(&hand_start_round) {
                    message = "You need to pass before saving".to_string();
                } else {
                    return true;
                }
            },
            "c" => {
                if !hand_start_round.contains(hand) {
                    message = "You can't pick a card until you've played all the cards you've taken from the table!".to_string();
                } else if !hand.contains(&hand_start_round) {
                    message = "You can't pick a card after having played something".to_string();
                } else if custom_rule_jokers && hand.contains_joker() {
                    message = "Jokers must be played!".to_string();
                } else {
                    match pick_a_card(hand, deck) {
                        Ok(card) => println!("You have picked a {}\x1b[38;2;0;0;0;1m", &card),
                        Err(_) => println!("No more card to draw!")
                    };
                    break
                }
            },
            "p" => {
                message = play_sequence(hand, table);
                print_situation(table, hand, deck);
            },
            "t" => {
                message = take_sequence(table, hand);
                print_situation(table, hand, deck);
            },
            "a" => {
                if !hand_start_round.contains(hand) {
                    message = "You can't pass until you've played all the cards you've taken from the table!".to_string();
                } else if hand.contains(&hand_start_round) {
                    message = "You need to play something to pass".to_string();
                } else if custom_rule_jokers && hand.contains_joker() {
                    message = "Jokers need to be played!".to_string();
                } else {
                    break
                }
            }
            "r" => {
                hand.sort_by_rank();
                print_situation(table, hand, deck);
            },
            "s" => {
                hand.sort_by_suit();
                print_situation(table, hand, deck);
            },
            "g" => {
                give_up(table, hand, deck, &hand_start_round, &table_start_round, &mut Sequence::new());
                print_situation(table, hand, deck);
            },
            _ => ()
        };
    }

    false
}


fn print_situation(table: &Table, hand: &Sequence, deck: &Sequence) {
    
    println!("\n{} cards remaining in the deck", deck.number_cards());
    
    // print the table
    println!("Table: \n{table}");

    // print the player hand
    println!("Your hand:\n{hand}\n");
    reset_style();

}


pub fn situation_to_string(table: &Table, hand: &Sequence, 
                           cards_from_table: &Sequence, message: &str) -> String {
  
    let hi = hand.show_indices();
    let ht = cards_from_table.show_indices_shifted(hand.number_cards());
    if cards_from_table.number_cards() == 0 {
        format!("\n{}\n{}\n{}{}:\n{}\n{}{}\n",
                "Table:", table, "Your hand", message, hi.0, reset_style_string(), hi.1)
    } else {
        format!("\n{}\n{}\n{}{}:\n{}{}\n{}\n\n{}\n{}\n{}{}\n", 
                "Table:", table, "Your hand", message, hi.0, reset_style_string(), hi.1,
                "Cards from the table:", ht.0, reset_style_string(), ht.1)
    }
}


pub fn get_input() -> Result<String, InvalidInputError> {
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => (),
        Err(_) => return Err(InvalidInputError {})
    }
    Ok(buffer)
}


fn pick_a_card(hand: &mut Sequence, deck: &mut Sequence) -> Result<Card, NoMoreCards> {
    let card = match deck.draw_card() {
        Some(c) => c,
        None => return Err(NoMoreCards {})
    };
    hand.add_card(card.clone());
    Ok(card)
}


fn play_sequence(hand: &mut Sequence, table: &mut Table) -> String {
    println!("Please enter the sequence, separated by spaces");
    let hand_and_indices = hand.show_indices();
    println!("{}", hand_and_indices.0);
    reset_style();
    println!("{}", hand_and_indices.1);
    let mut seq = Sequence::new();
    
    let mut s = get_input().unwrap_or_else(|_| {"".to_string()});
    s.pop();
    let mut seq_i = Vec::<usize>::new();
    for item in s.split(' ') {
        if let Ok(n) = item.parse::<usize>() {
            let mut n_i = 0;
            for &i in &seq_i {
                if i < n {
                    n_i += 1;
                }
            }
            let card = match hand.take_card(n-n_i) {
                Some(c) => c,
                None => continue
            };
            seq.add_card(card);
            seq_i.push(n);
        }
    }

    if seq.is_valid() {
        table.add(seq);
        String::new()
    } else {
        let message = format!("{} is not a valid sequence!", &seq);
        hand.merge(seq);
        message
    }
}


fn take_sequence(table: &mut Table, hand: &mut Sequence) -> String {
    println!("Which sequence would you like to take?");
    match get_input().unwrap_or_else(|_| {"".to_string()})
          .trim().parse::<usize>() {
        Ok(n) => match table.take(n) {
            Some(seq) => {
                hand.merge(seq);
                String::new()
            },
            None => "This sequence is not on the table".to_string()
        },
        Err(_) => "Error parsing the input!".to_string()
    }
}


pub fn give_up(table: &mut Table, hand: &mut Sequence, deck: &mut Sequence, 
               hand_start_round: &Sequence, table_start_round: &Table,
               cards_from_table: &mut Sequence) {
    
    // reset the situation
    *hand = hand_start_round.clone();
    *table = table_start_round.clone();
    *cards_from_table = Sequence::new();

    // penalty
    for _i in 0..PENALTY_RESET {
        match pick_a_card(hand, deck) {
            Ok(_) => (),
            Err(_) => {
                println!("No more card to draw!");
                break;
            }
        };
    }
}


/// convert the game info to a sequence of bytes
pub fn game_to_bytes (starting_player: u8, player: u8, table: &Table, hands: &[Sequence], 
                      deck: &Sequence, config: &Config, player_names: &[String]) -> Vec<u8> {
    
    // construct the sequence of bytes to be saved
    let mut bytes = Vec::<u8>::new();
    
    // config
    bytes.append(&mut config.to_bytes());

    // starting player
    bytes.push(starting_player);
    
    // player about to play
    bytes.push(player);
    
    // hand of each player
    for i_player in 0..config.n_players {
        
        // number of cards in the hand as 2 u8
        let n_cards_in_hand = hands[i_player as usize].number_cards() as u16;
        bytes.push((n_cards_in_hand >> 8) as u8);
        bytes.push((n_cards_in_hand & 255) as u8);
        
        // append the hand
        bytes.append(&mut hands[i_player as usize].to_bytes());
    }

    // player names
    for i_player in 0..config.n_players {
        let name_b = player_names[i_player as usize].as_bytes();
        bytes.push(name_b.len() as u8);
        bytes.append(&mut name_b.to_vec());
    }
    
    // deck 
    let n_cards_in_deck = deck.number_cards();
    bytes.push((n_cards_in_deck >> 8) as u8);
    bytes.push((n_cards_in_deck & 255) as u8);
    bytes.append(&mut deck.to_bytes());
    
    // table 
    bytes.append(&mut table.to_bytes());

    bytes
}


/// load the game info from a sequence of bytes
#[allow(clippy::type_complexity)]
pub fn load_game(bytes: &[u8]) -> Result<(Config, u8, u8, Table, Vec<Sequence>, Sequence, Vec<String>), LoadingError> {
    let mut i_byte: usize = 0; // index of the current element in bytes

    // load the config
    let n_bytes_config: usize = 6;
    let config = Config::from_bytes(&bytes[i_byte..n_bytes_config]);
    i_byte += n_bytes_config;
    
    // load the starting player
    let starting_player = bytes[i_byte];
    i_byte += 1;
    
    // load the current player
    let player = bytes[i_byte];
    i_byte += 1;
    
    // hand of each player
    let mut hands = Vec::<Sequence>::new();
    for _i_player in 0..config.n_players {
        
        // number of cards in the hand as 2 u8
        let n_cards_in_hand = ((bytes[i_byte] as usize) << 8) + (bytes[i_byte+1] as usize);
        i_byte += 2;
 
        // append the hand
        hands.push(Sequence::from_bytes(&bytes[i_byte..i_byte+n_cards_in_hand]));
        i_byte += n_cards_in_hand;
    }
    
    // player names
    let mut player_names = Vec::<String>::new();
    for i_player in 0..config.n_players {
        
        // number of characters in the name
        let n_chars = bytes[i_byte] as usize;
        i_byte += 1;
        
        // append the name
        player_names.push(String::from_utf8(bytes[i_byte..i_byte+n_chars].to_vec())
                          .unwrap_or_else(|_| {format!("Player {}", i_player+1)}));
        i_byte += n_chars;
    }

    // deck
    let n_cards_in_deck = ((bytes[i_byte] as usize) << 8) + (bytes[i_byte+1] as usize);
    i_byte += 2;
    let deck = Sequence::from_bytes(&bytes[i_byte..i_byte+n_cards_in_deck]);
    i_byte += n_cards_in_deck;

    // table
    let table = Table::from_bytes(&bytes[i_byte..]);

    Ok((
        config,
        starting_player,
        player,
        table,
        hands,
        deck,
        player_names
    ))
}


#[derive(Debug)]
pub struct InvalidInputError {}

impl<T: std::error::Error> From<T> for InvalidInputError {
    fn from(_error: T) -> Self {
        InvalidInputError {}
    }
}

pub struct NoMoreCards {}
pub struct LoadingError {}


