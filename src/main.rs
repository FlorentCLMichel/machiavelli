//! # Machiavelli
//!
//! A simple machiavelli card game *(work in progress)*

use std::process;
use std::io::{ stdin, Read, Write };
use std::fs::File;
use rand::rng;
use machiavelli::*;

fn main() {

    // set the style
    reset_style();

    // clear the terminal
    print!("\x1b[2J\x1b[1;1H");

    // get the config
    println!("Hi there! Up for a game of Machiavelli?\n");
    let mut config = match get_config() {
        Ok(conf) => conf, 
        Err(_) => {
            println!("Invalid input!");
            process::exit(1);
        },
    };
    
    // create the table
    let mut table = Table::new();
    let mut deck = Sequence::new();
    let mut hands = Vec::<Sequence>::new();
    let mut starting_player: u8 = 0;
    let mut player: u8 = 0;
    let mut player_names = Vec::<String>::new();

    if config.n_decks == 0 {
        
        // load the previous game
        println!("Name of the save file:");
        let mut fname = String::new();
        let mut bytes = Vec::<u8>::new();
        let mut retry = true;
        while retry {

            retry = false;
            
            // get the file name
            match stdin().read_line(&mut fname) {
                Ok(_) => (),
                Err(_) => retry = true
            };

            fname = fname.trim().to_string();

            if !retry {

                // load the data from the file
                let mut file: File; 
                match File::open(fname.clone()) {
                    Ok(f) => file = f,
                    Err(_) => {
                        println!("Could not open the file!");
                        retry = true;
                        fname.clear();
                        continue;
                    }
                };
                match file.read_to_end(&mut bytes) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Could not read from the file!");
                        retry = true;
                        bytes.clear();
                        fname.clear();
                    }
                };
                
                // decode the sequence of bytes
                bytes = encode::xor(&bytes, fname.as_bytes());

                match load_game(&bytes) {
                    Ok(lg) => {
                        config = lg.0;
                        starting_player = lg.1; 
                        player = lg.2; 
                        table = lg.3;
                        hands = lg.4; 
                        deck = lg.5;
                        player_names = lg.6;
                        bytes = Vec::<u8>::new();
                    },
                    Err(_) => {
                        println!("Error loading the save file!");
                    }
                };
            }
        }

    } else {

        // build the deck
        let mut rng = rng();
        deck = Sequence::multi_deck(config.n_decks, config.n_jokers, &mut rng);
        
        // build the hands
        hands = vec![Sequence::new(); config.n_players as usize];
        for i in 0..config.n_players {
            for _ in 0..config.n_cards_to_start {
                hands[i as usize].add_card(deck.draw_card().unwrap());
            }
        }

        // get the players name
        for i in 0..config.n_players {
            println!("Player {}'s name: ", i+1);
            let mut cont = true;
            while cont {
                match get_input() {
                    Ok(s) => {
                        player_names.push(s.trim().to_string());
                        cont = false
                    },
                    Err(_) => println!("Could not parse the input")
                };
            }
        }


    }
    
    // play until a player wins, there is no card left in the deck, or the player decides to save
    // and quit
    let mut save_and_quit: bool;
    loop {
        if deck.number_cards() == 0 {
            println!("\x1b[1mNo more cards in the deck—It's a draw!\x1b[0m\n");
            break;
        }
        save_and_quit = player_turn(&mut table, &mut hands[player as usize], 
                                    &mut deck, config.custom_rule_jokers, &player_names[player as usize]);
        if save_and_quit {
            
            // convert the game data to a sequence of bytes
            let mut bytes = game_to_bytes(starting_player, player, &table, &hands, &deck, &config, &player_names);

            println!("Name of the save file:");
            let mut fname = String::new();
            let mut retry = true;
            while retry {

                retry = false;
                
                // get the file name
                match stdin().read_line(&mut fname) {
                    Ok(_) => (),
                    Err(_) => retry = true
                };
                fname = fname.trim().to_string();

                // obfuscate the save file (not very secure!)
                bytes = encode::xor(&bytes, fname.as_bytes());
                
                if !retry {

                    // save the data to the file
                    let mut file: File; 
                    match File::create(fname.clone()) {
                        Ok(f) => file = f,
                        Err(_) => {
                            println!("Could not create the file!");
                            retry = true;
                            continue;
                        }
                    };
                    match file.write_all(&bytes) {
                        Ok(_) => (),
                        Err(_) => {
                            println!("Could not write to the file!");
                            retry = true;
                        }
                    };
                }
            }

            break;
        }
        if hands[player as usize].number_cards() == 0 {
            println!("\x1b[1mPlayer {} wins! Congratulations!\x1b[0m\n", player+1);
            break;
        }
        player = (player + 1) % config.n_players;
    }
    
    // reset the style
    println!("\x1b[0m");
    print!("\x1b[?25h");
}
