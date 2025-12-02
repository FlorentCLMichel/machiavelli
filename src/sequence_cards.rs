//! Define representations for cards and sequences of cards.

use std::fmt;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use crate::sort::sort;
pub use Card::*;
pub use Suit::*;

static MAX_VAL: u8 = 13;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Suit {
    Heart,
    Diamond,
    Club,
    Spade
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Card {
    RegularCard(Suit, u8),
    Joker
}

fn suit_to_int(suit: Suit) -> u8 {
    match suit {
        Heart => 1,
        Club => 2,
        Diamond => 3,
        Spade => 4,
    }
}

fn int_to_suit(s: u8) -> Option<Suit> {
    match s {
        1 => Some(Heart),
        2 => Some(Club),
        3 => Some(Diamond),
        4 => Some(Spade),
        _ => None
    }
}

impl Card {

    fn from_byte(x: u8) -> Option<Card> {
        if x == 0 {
            return Some(Joker);
        }
        let mut val = x % MAX_VAL;
        if val == 0 {
            val = MAX_VAL;
        }
        int_to_suit((x-1) / MAX_VAL + 1).map(|suit| RegularCard(suit, val))
    }

    fn to_byte(&self) -> u8 {
        match self {
            Joker => 0,
            RegularCard(suit, value) => (suit_to_int(*suit)-1) * MAX_VAL + value
        }
    }

}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegularCard(suit, val) => {
                let str_val = match val {
                    1 => "A".to_string(),
                    11 => "J".to_string(),
                    12 => "Q".to_string(),
                    13 => "K".to_string(),
                    10 => "10".to_string(),
                    _ => format!("{val}")
                };
                let char_suit = match suit {
                    Heart => '♥',
                    Diamond => '♦',
                    Club => '♣',
                    Spade => '♠',
                };
                let color = match suit {
                    Heart => "31",
                    Diamond => "31",
                    Club => "30",
                    Spade => "30",
                };
                write!(f, "\x1b[1;{color}m{str_val}{char_suit}")
            },
            Joker => write!(f, "\x1b[1;34m#")
        }
    }
}

/// Sequence of cards
#[derive(Debug, PartialEq, Clone)]
pub struct Sequence(Vec<Card>);

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

impl Sequence {

    /// Create an empty sequence of cards
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::Sequence;
    ///
    /// let sequence = Sequence::new();
    ///
    /// assert_eq!(0, sequence.number_cards());
    /// ```
    pub fn new() -> Sequence {
        Sequence(Vec::<Card>::new())
    }

    /// Create a sequence from an array of cards
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3)
    /// ];
    /// let sequence = Sequence::from_cards(&cards);
    ///
    /// assert_eq!(4, sequence.number_cards());
    /// ```
    pub fn from_cards(cards: &[Card]) -> Sequence {
        Sequence(cards.to_vec())
    }
    
    /// Create a sequence from an array of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let bytes: [u8; 5] = [0, 1, 20, 35, 51];
    /// let sequence = Sequence::from_bytes(&bytes);
    ///
    /// assert_eq!(Sequence::from_cards(&[
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Club, 7),
    ///     RegularCard(Diamond, 9),
    ///     RegularCard(Spade, 12),
    /// ]), sequence);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Sequence {
        let mut cards = Vec::<Card>::new(); 
        for byte_ptr in bytes {
            if let Some(card) = Card::from_byte(*byte_ptr) {
                cards.push(card);
            }
        }
        Sequence::from_cards(&cards)
    }
    
    /// Convert a sequence to an array of bytes
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Diamond, 7),
    ///     RegularCard(Club, 9),
    ///     RegularCard(Spade, 12),
    /// ];
    /// let sequence = Sequence::from_cards(&cards);
    ///
    /// let bytes = sequence.to_bytes();
    ///
    /// assert_eq!(vec![0, 1, 33, 22, 51], bytes);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::<u8>::new();
        for card in &self.0 {
            res.push(card.to_byte());
        }
        res
    }

    /// Return the number of cards in the sequence
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let sequence = Sequence::from_cards(&cards);
    ///
    /// assert_eq!(5, sequence.number_cards());
    /// ```
    pub fn number_cards(&self) -> usize {
        self.0.len()
    }
    
    /// Return a string with the indices
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 10),
    ///     Joker,
    ///     RegularCard(Club, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Club, 3),
    ///     RegularCard(Club, 4),
    ///     RegularCard(Club, 10),
    ///     RegularCard(Club, 6),
    ///     RegularCard(Club, 10),
    /// ];
    /// let sequence = Sequence::from_cards(&cards);
    ///
    /// assert_eq!(sequence.show_indices(), 
    ///            ("\u{1b}[1;34m# \u{1b}[1;31mA♥ \u{1b}[1;31m3♥ \u{1b}[1;30m10♣ \u{1b}[1;34m# \u{1b}[1;30mA♣ \u{1b}[1;31m2♥ \u{1b}[1;30m3♣ \u{1b}[1;30m4♣ \u{1b}[1;30m10♣ \u{1b}[1;30m6♣ \u{1b}[1;30m10♣".to_string(),
    ///             "1 2  3  4   5 6  7  8  9  10  11 12".to_string()));
    /// ```
    pub fn show_indices(&self) -> (String,String) {

        let mut first_line = String::new();
        let mut second_line = String::new();
        let mut n_chars_1: usize;
        let mut n_chars_2: usize = 2;
        let mut power_of_ten: usize = 10;
        for i in 1..=self.0.len() {
            
            // if i is a power of 10, increase the number of characters for the second line by 1
            if i==power_of_ten {
                n_chars_2 += 1;
                power_of_ten *= 10;
            }
            
            // print the current card with a space
            let current_card = &self.0[i-1];
            first_line.push_str(&format!("{current_card} "));
            
            // see how many characters the current caerd take
            match current_card {
                Joker => n_chars_1 = 2,
                RegularCard(_,10) => n_chars_1 = 4,
                _ => n_chars_1 = 3
            };

            // print the index
            second_line.push_str(&format!("{i} "));

            // pad the first line with spaces if necessary
            for _ in n_chars_1..n_chars_2 {
                first_line.push(' ');
            }
            
            // pad the second line with spaces if necessary
            for _ in n_chars_2..n_chars_1 {
                second_line.push(' ');
            }
        }
        
        first_line = first_line.trim().to_string();
        second_line = second_line.trim().to_string();
        (first_line.to_string(), second_line.to_string())
    }
    
    /// Return a string with the indices shifted by `n`
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 10),
    ///     Joker,
    ///     RegularCard(Club, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Club, 3),
    ///     RegularCard(Club, 4),
    ///     RegularCard(Club, 10),
    ///     RegularCard(Club, 6),
    ///     RegularCard(Club, 10),
    /// ];
    /// let sequence = Sequence::from_cards(&cards);
    ///
    /// assert_eq!(sequence.show_indices_shifted(1), 
    ///            ("\u{1b}[1;34m# \u{1b}[1;31mA♥ \u{1b}[1;31m3♥ \u{1b}[1;30m10♣ \u{1b}[1;34m# \u{1b}[1;30mA♣ \u{1b}[1;31m2♥ \u{1b}[1;30m3♣ \u{1b}[1;30m4♣ \u{1b}[1;30m10♣ \u{1b}[1;30m6♣ \u{1b}[1;30m10♣".to_string(),
    ///             "2 3  4  5   6 7  8  9  10 11  12 13".to_string()));
    /// ```
    pub fn show_indices_shifted(&self, n: usize) -> (String,String) {

        let mut first_line = String::new();
        let mut second_line = String::new();
        let mut n_chars_1: usize;
        let mut n_chars_2: usize = 2;
        let mut power_of_ten: usize = 10;
        while power_of_ten <= n {
            power_of_ten *= 10;
            n_chars_2 += 1;
        }
        
        for i in (n+1)..=(n+self.0.len()) {
            
            // if i is a power of 10, increase the number of characters for the second line by 1
            if i==power_of_ten {
                n_chars_2 += 1;
                power_of_ten *= 10;
            }
            
            // print the current card with a space
            let current_card = &self.0[i-n-1];
            first_line.push_str(&format!("{current_card} "));
            
            // see how many characters the current card takes
            match current_card {
                Joker => n_chars_1 = 2,
                RegularCard(_,10) => n_chars_1 = 4,
                _ => n_chars_1 = 3
            };

            // print the index
            second_line.push_str(&format!("{i} "));

            // pad the first line with spaces if necessary
            for _ in n_chars_1..n_chars_2 {
                first_line.push(' ');
            }
            
            // pad the second line with spaces if necessary
            for _ in n_chars_2..n_chars_1 {
                second_line.push(' ');
            }
        }
        
        first_line = first_line.trim().to_string();
        second_line = second_line.trim().to_string();
        (first_line.to_string(), second_line.to_string())
    }
    
    /// Sort cards by suit
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11),
    ///     RegularCard(Club, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Club, 3)
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    /// sequence.sort_by_suit();
    ///
    /// assert_eq!(
    ///     Sequence::from_cards(&[
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 1),
    ///     RegularCard(Club, 3),
    ///     RegularCard(Club, 11),
    ///     Joker
    ///     ]),
    ///     sequence);
    /// ```
    pub fn sort_by_suit(&mut self) {
        self.0 = sort(&self.0, Box::new(value_card_by_suit));
    }
    

    /// Sort cards by rank
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11),
    ///     RegularCard(Club, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Club, 3)
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    /// sequence.sort_by_rank();
    ///
    /// assert_eq!(
    ///     Sequence::from_cards(&[
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Club, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 3),
    ///     RegularCard(Club, 11),
    ///     Joker
    ///     ]),
    ///     sequence);
    /// ```
    pub fn sort_by_rank(&mut self) {
        self.0 = sort(&self.0, Box::new(value_card_by_rank));
    }
    
    /// Reverse the sequence 
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let mut sequence_1 = Sequence::from_cards(&cards);
    /// let sequence_2 = sequence_1.reverse();
    ///
    /// assert_eq!(Sequence::from_cards(&[
    ///     RegularCard(Club, 11),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Heart, 2)
    /// ]), sequence_2);
    /// ```
    pub fn reverse(mut self) -> Sequence {
        let mut res = Sequence::new();
        while let Some(card) = self.draw_card() {
            res.add_card(card);
        }
        res
    }

    /// Merge the sequence with another one (in reversed order)
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards_1 = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    /// ];
    /// let cards_2 = [
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let mut sequence_1 = Sequence::from_cards(&cards_1);
    /// let sequence_2 = Sequence::from_cards(&cards_2);
    ///
    /// sequence_1.merge(sequence_2);
    ///
    /// assert_eq!(5, sequence_1.number_cards());
    /// ```
    pub fn merge(&mut self, mut seq: Sequence) {
        while let Some(card) = seq.draw_card() {
            self.add_card(card);
        }
    }

    /// Build a randomly-shuffled deck of cards
    ///
    /// # Arguments
    ///
    /// * `n_decks`: the number of copies of a full deck of 52 cards
    /// * `n_jokers`: the number of jokers
    /// * `rng`: mutable reference to the random-number generator used foor shuffling
    ///
    /// # Example
    ///
    /// ```
    /// use rand::rng;
    /// use machiavelli::sequence_cards::Sequence;
    ///
    /// let mut rng = rng();
    /// let sequence = Sequence::multi_deck(3, 6, &mut rng);
    ///
    /// assert_eq!(162, sequence.number_cards());
    /// ```
    pub fn multi_deck(n_decks: u8, n_jokers: u8, rng: &mut ThreadRng) -> Sequence {
        
        let mut deck = Sequence::new();

        for _i in 0..n_decks {

            // add the regular cards
            for val in 1..=MAX_VAL {
                for suit in &[Heart, Diamond, Club, Spade] {
                    deck.add_card(RegularCard(*suit, val));
                }
            }
        }
            
        // add the jokers
        for _j in 0..n_jokers {
            deck.add_card(Joker);
        }

        // shuffle the deck
        deck.shuffle(rng);

        deck
    }
    
    /// Add a card to a sequence
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    /// sequence.add_card(Joker);
    ///
    /// assert_eq!(6, sequence.number_cards());
    /// ```
    pub fn add_card(&mut self, card: Card) {
        self.0.push(card);
    }
    
    /// Draw the top card from a sequence
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    /// let card = sequence.draw_card().unwrap();
    ///
    /// assert_eq!(4, sequence.number_cards());
    /// assert_eq!(RegularCard(Club, 11), card);
    /// ```
    pub fn draw_card(&mut self) -> Option<Card> {
        self.0.pop()
    }
    
    /// Take a card from a sequence
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     Joker, 
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    /// let card = sequence.take_card(2).unwrap();
    ///
    /// assert_eq!(4, sequence.number_cards());
    /// assert_eq!(RegularCard(Heart, 1), card);
    /// ```
    pub fn take_card(&mut self, i: usize) -> Option<Card> {
        if (i>0) && (i<= self.0.len()) {
            let card = self.0[i-1].clone();
            self.0.remove(i-1);
            return Some(card);
        } 
        None
    }
    
    /// Check if a sequence has a joker
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards_1 = Sequence::from_cards(&[
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     Joker, 
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ]);
    /// let cards_2 = Sequence::from_cards(&[
    ///     RegularCard(Heart, 1),
    ///     RegularCard(Heart, 2),
    ///     RegularCard(Heart, 3),
    ///     RegularCard(Club, 11)
    /// ]);
    ///
    /// assert_eq!(true, cards_1.contains_joker());
    /// assert_eq!(false, cards_2.contains_joker());
    /// ```
    pub fn contains_joker(&self) -> bool {
        for card in &self.0 {
            if *card == Joker {
                return true;
            }
        }
        false
    }

    /// Check if a sequence if valid for the Machiavelli game
    ///
    /// # Example
    ///
    /// ```
    /// use machiavelli::sequence_cards::{ Sequence, Card::* , Suit::*};
    ///
    /// let cards = [
    ///     RegularCard(Heart, 1),
    ///     Joker, 
    ///     RegularCard(Heart, 3),
    /// ];
    /// let mut sequence = Sequence::from_cards(&cards);
    ///
    /// assert_eq!(sequence.is_valid(), true);
    /// ```
    pub fn is_valid(&mut self) -> bool {
        
        if self.0.is_empty() {
            return false;
        }
        
        if self.has_only_jokers() {
            return true;
        }

        if self.0.len() < 3 {
            return false;
        }

        // sort the equence
        self.sort_by_rank();
     
        if self.is_valid_sequence_same_val() {
            return true;
        }
        
        if self.is_valid_sequence_same_suit() {
            return true;
        }
        
        // if the first card is an ace, also try with the ace at the end
        if let RegularCard(_, 1) = self.0[0] {
            let ace = self.0[0].clone();
            self.0 = self.0[1..].to_vec();
            self.0.push(ace);
        };
        
        if self.is_valid_sequence_same_suit() {
            return true;
        }
 
        false
    }

    /// return the vector of cards
    pub fn to_vec(&self) -> Vec<Card> {
        self.0.clone()
    }

    /// determine if the sequence contains another one
    pub fn contains(&self, seq: &Sequence) -> bool {
        let count_rhs = seq.count_cards();
        let count_self = self.count_cards();
        for (card, count) in count_rhs {
            if !count_self.contains_key(&card) {
                return false;
            }
            if count_self[&card] < count {
                return false;
            }
        }
        true
    }

    // randomly shuffle the sequence
    fn shuffle(&mut self, rng: &mut ThreadRng) {
        self.0.shuffle(rng);
    }
        
    // check if the sequence contains only jokers
    fn has_only_jokers(&self) -> bool {
        for card in &self.0 {
            if *card != Joker {
                return false;
            }
        }
        true
    }

    // /// count the number of jokers in the sequence
    // fn n_jokers(&self) -> u8 {
    //     let mut res = 0;
    //     for card in &self.0 {
    //         match card {
    //             Joker => res += 1,
    //             _ => ()
    //         };
    //     }
    //     res
    // }

    // check if the sequence is a valid one with a unique card value
    fn is_valid_sequence_same_val(&self) -> bool {
        let mut suits_in_seq = Vec::<Suit>::new();
        let mut common_value: u8 = 0;
        for card in &self.0 {
            match card {
                RegularCard(suit, value) => {
                    if common_value == 0 {
                        common_value = *value;
                    }
                    else if (suits_in_seq.contains(suit)) || (*value != common_value) {
                        return false
                    }
                    suits_in_seq.push(*suit);
                }
                Joker => ()
            }
        }
        true
    }

    // check if the sequence is a valid one with a unique suit
    fn is_valid_sequence_same_suit(&mut self) -> bool {
        let mut jokers = self.take_jokers();
        let mut common_suit = Club;
        let mut current_value: u8 = 0;
        let mut di: usize = 0;
        for i in 0..self.0.len() {
            let card = &self.0[i+di];
            match card {
                RegularCard(suit, value) => {
                    if current_value == 0 {
                        common_suit = *suit;
                        current_value = *value;
                    } else {
                        if *suit != common_suit {
                            self.merge(jokers);
                            return false
                        }
                        if (*value != current_value + 1)
                              &&
                              ((current_value < MAX_VAL) || (*value != 1))
                        {
                            let next_val = match *value {
                                1 => MAX_VAL + 1,
                                n => n
                            };
                            if next_val < (current_value + 1) {
                                self.merge(jokers);
                                return false;
                            }
                            let diff = next_val - current_value - 1;
                            current_value += diff;
                            if (diff as usize) > jokers.number_cards() {
                                self.merge(jokers);
                                return false;
                            }
                            for _ in 0..diff {
                                di += 1;
                                self.0.insert(i+di-1, jokers.draw_card().unwrap());
                            }
                        }
                        current_value += 1;
                    }
                }
                Joker => ()
            }
        }
        self.merge(jokers);
        
        // avoid configurations like KA#..., K##..., which look wrong
        match self.0[0] {
            RegularCard(_, val) if val == MAX_VAL => {
                match self.0[1] {
                    Joker => { 
                        self.0.swap(0,1); 
                    },
                    RegularCard(_, 1) => {
                        if self.0[2] == Joker {
                            self.0.swap(1,2);
                            self.0.swap(0,1);
                        }
                    },
                    _ => ()
                }
            }
            _ => ()
        }
        
        true
    }

    /// take the jokers from the sequence and return them as a separate sequence
    fn take_jokers(&mut self) -> Sequence {
        let mut res = Sequence::new();
        let mut di: usize = 0;
        for i in 0..self.number_cards() {
            if self.0[i-di] == Joker {
                res.add_card(self.take_card(i+1-di).unwrap());
                di += 1;
            }
        }
        res
    }

    /// count the numbers of cards of each type and return them as a hashmap
    fn count_cards(&self) -> HashMap<Card, u16> {
        let mut res = HashMap::<Card, u16>::new();
        
        for card in &self.0 {
            if res.contains_key(card) {
                *res.get_mut(card).unwrap() += 1;
            } else {
                res.insert(card.clone(), 1);
            }
        }
        
        res
    }

}


impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for card in &self.0 {
            card.fmt(f)?;
            write!(f, " ")?;
        }
        write!(f, "")
    }
}


/// assign a value to each card, with the suit given higher weight than the value
fn value_card_by_suit(card: &Card) -> u8 {
    match *card {
        Joker => 255,
        RegularCard(suit, val) => (MAX_VAL + 1) * suit_to_int(suit) + val
    }
}


/// assign a value to each card, with the value given higher weight than the suit
fn value_card_by_rank(card: &Card) -> u8 {
    match *card {
        Joker => 255,
        RegularCard(suit, val) => 4 * val + suit_to_int(suit)
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use Card::{ RegularCard, Joker };
    use rand::rng;

    #[test]
    fn take_jokers_1() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker
        ]);
        assert_eq!(seq.take_jokers(), 
                   Sequence::from_cards(&[
                       Joker,
                       Joker
                   ]));
    }
    
    #[test]
    fn take_jokers_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 4), 
            RegularCard(Heart, 5), 
        ]);
        assert_eq!(seq.take_jokers(), 
                   Sequence::new());
    }
    
    #[test]
    fn take_jokers_3() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 4), 
            Joker,
            RegularCard(Heart, 5), 
        ]);
        assert_eq!(seq.take_jokers(), 
                   Sequence::from_cards(&[
                       Joker
                   ]));
    }
    
    #[test]
    fn sequence_two_jokers() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker
        ]);
        assert_eq!(seq.is_valid(), true);
    }

    #[test]
    fn sequence_three_jokers() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker, 
            Joker
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_only_jokers_1() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker, 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_only_jokers_2() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker, 
            Joker, 
            Joker, 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 1), 
            RegularCard(Heart, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 1), 
            RegularCard(Club, 2), 
            RegularCard(Club, 3), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_3() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            RegularCard(Club, 3), 
            RegularCard(Club, 4), 
            RegularCard(Club, 5), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_4() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            RegularCard(Club, 3), 
            RegularCard(Club, 5), 
            RegularCard(Club, 6), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_5() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            RegularCard(Club, 3), 
            RegularCard(Club, 3), 
            RegularCard(Club, 4), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_6() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 12), 
            RegularCard(Club, 13), 
            RegularCard(Club, 1), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_7() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 13), 
            RegularCard(Club, 1), 
            RegularCard(Club, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_8() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 1), 
            RegularCard(Heart, 2), 
            RegularCard(Heart, 3), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_9() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Heart, 3), 
            RegularCard(Heart, 4), 
            RegularCard(Heart, 5), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_10() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Heart, 3), 
            RegularCard(Heart, 5), 
            RegularCard(Heart, 6), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_11() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Heart, 3), 
            RegularCard(Heart, 3), 
            RegularCard(Heart, 4), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_12() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 12), 
            RegularCard(Heart, 13), 
            RegularCard(Heart, 1), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_13() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 13), 
            RegularCard(Heart, 1), 
            RegularCard(Heart, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_14() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            RegularCard(Club, 9), 
            RegularCard(Club, 10), 
            RegularCard(Club, 12), 
            RegularCard(Club, 13), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_15() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            RegularCard(Club, 8), 
            RegularCard(Club, 9), 
            RegularCard(Club, 12), 
            Joker, 
            RegularCard(Club, 13), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_16() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 8), 
            RegularCard(Club, 9), 
            RegularCard(Club, 12), 
            Joker, 
            RegularCard(Club, 13), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_one_j_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Diamond, 2), 
            RegularCard(Diamond, 3), 
            Joker, 
            RegularCard(Diamond, 5), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_suit_one_j_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Diamond, 2), 
            RegularCard(Diamond, 3), 
            Joker, 
            RegularCard(Diamond, 6), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_suit_two_j_1() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            RegularCard(Diamond, 3), 
            Joker, 
            RegularCard(Diamond, 5), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_val_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Diamond, 2), 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_val_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Diamond, 2), 
            RegularCard(Spade, 2), 
            RegularCard(Club, 2), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_val_3() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            RegularCard(Spade, 2), 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_val_one_j_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            Joker, 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_val_one_j_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Heart, 2), 
            Joker, 
            RegularCard(Heart, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn sequence_same_val_two_j_1() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            Joker, 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn sequence_same_val_two_j_2() {
        let mut seq = Sequence::from_cards(&[
            Joker, 
            RegularCard(Club, 2), 
            Joker, 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), true);
    }
    
    #[test]
    fn invalid_sequence_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            RegularCard(Spade, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn invalid_sequence_2() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            RegularCard(Diamond, 3), 
            RegularCard(Heart, 2), 
        ]);
        assert_eq!(seq.is_valid(), false);
    }
    
    #[test]
    fn seq_with_joker_1() {
        let mut cards = Sequence::from_cards(&[
            RegularCard(Heart, 1),
            RegularCard(Heart, 3),
            Joker, 
        ]);
        
        assert_eq!(true, cards.is_valid());
    }
    
    #[test]
    fn seq_with_joker_2() {
        let mut cards = Sequence::from_cards(&[
            RegularCard(Heart, 1),
            RegularCard(Spade, 3),
            Joker, 
        ]);
        
        assert_eq!(false, cards.is_valid());
    }
    
    #[test]
    fn seq_with_joker_3() {
        let mut cards = Sequence::from_cards(&[
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 11),
        ]);
        
        assert_eq!(false, cards.is_valid());
    }
    
    #[test]
    fn seq_with_joker_4() {
        let mut cards = Sequence::from_cards(&[
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 12),
        ]);
        
        assert_eq!(true, cards.is_valid());
    }
    
    #[test]
    fn seq_with_joker_5() {
        let mut cards = Sequence::from_cards(&[
            Joker, 
            RegularCard(Heart, 4),
            RegularCard(Heart, 7),
        ]);
        
        assert_eq!(false, cards.is_valid());
    }
    
    #[test]
    fn number_cards_1() {
        let seq = Sequence::new();
        assert_eq!(0, seq.number_cards());
    }
    
    #[test]
    fn number_cards_2() {
        let seq = Sequence::from_cards(&[
            Joker, 
            RegularCard(Heart, 2)
        ]);
        assert_eq!(2, seq.number_cards());
    }
    
    #[test]
    fn number_cards_3() {
        let seq = Sequence::from_cards(&[
            Joker, 
            Joker, 
            RegularCard(Heart, 2),
            RegularCard(Heart, 2)
        ]);
        assert_eq!(4, seq.number_cards());
    }

    #[test]
    fn build_deck_1() {
        let mut rng = rng();
        let deck = Sequence::multi_deck(1, 0, &mut rng);
        assert_eq!(52, deck.number_cards());
    }
    
    #[test]
    fn build_deck_2() {
        let mut rng = rng();
        let deck = Sequence::multi_deck(2, 4, &mut rng);
        assert_eq!(108, deck.number_cards());
    }
    
    #[test]
    fn build_deck_3() {
        let mut rng = rng();
        let deck = Sequence::multi_deck(3, 1, &mut rng);
        assert_eq!(157, deck.number_cards());
    }
    
    #[test]
    fn display_sequence_1() {
        let seq = Sequence::from_cards(&[
            RegularCard(Club, 2), 
            Joker,
            RegularCard(Diamond, 3), 
            RegularCard(Heart, 2), 
        ]);
        assert_eq!("\u{1b}[1;30m2♣ \u{1b}[1;34m# \u{1b}[1;31m3♦ \u{1b}[1;31m2♥ ".to_string(), format!("{seq}"));
    }

    #[test]
    fn display_shifted_1() {
        let cards = [
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 3),
            RegularCard(Club, 10),
            Joker,
            RegularCard(Club, 1),
            RegularCard(Heart, 2),
            RegularCard(Club, 3),
            RegularCard(Club, 4),
            RegularCard(Club, 10),
            RegularCard(Club, 6),
            RegularCard(Club, 10),
        ];
        let sequence = Sequence::from_cards(&cards);
       
        assert_eq!(sequence.show_indices_shifted(1), 
                   ("\u{1b}[1;34m# \u{1b}[1;31mA♥ \u{1b}[1;31m3♥ \u{1b}[1;30m10♣ \u{1b}[1;34m# \u{1b}[1;30mA♣ \u{1b}[1;31m2♥ \u{1b}[1;30m3♣ \u{1b}[1;30m4♣ \u{1b}[1;30m10♣ \u{1b}[1;30m6♣ \u{1b}[1;30m10♣".to_string(),
                    "2 3  4  5   6 7  8  9  10 11  12 13".to_string()));
    }
    
    #[test]
    fn display_shifted_2() {
        let cards = [
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 3),
            RegularCard(Club, 10),
            Joker,
            RegularCard(Club, 1),
            RegularCard(Heart, 2),
            RegularCard(Club, 3),
            RegularCard(Club, 4),
            RegularCard(Club, 10),
            RegularCard(Club, 6),
            RegularCard(Club, 10),
        ];
        let sequence = Sequence::from_cards(&cards);
       
        assert_eq!(sequence.show_indices_shifted(11), 
                   ("\u{1b}[1;34m#  \u{1b}[1;31mA♥ \u{1b}[1;31m3♥ \u{1b}[1;30m10♣ \u{1b}[1;34m#  \u{1b}[1;30mA♣ \u{1b}[1;31m2♥ \u{1b}[1;30m3♣ \u{1b}[1;30m4♣ \u{1b}[1;30m10♣ \u{1b}[1;30m6♣ \u{1b}[1;30m10♣".to_string(),
                    "12 13 14 15  16 17 18 19 20 21  22 23".to_string()));
    }
    
    #[test]
    fn display_shifted_3() {
        let cards = [
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 3),
            RegularCard(Club, 10),
            Joker,
            RegularCard(Club, 1),
            RegularCard(Heart, 2),
            RegularCard(Club, 3),
            RegularCard(Club, 4),
            RegularCard(Club, 10),
            Joker,
            RegularCard(Club, 10),
        ];
        let sequence = Sequence::from_cards(&cards);
       
        assert_eq!(sequence.show_indices_shifted(11), 
                   ("\u{1b}[1;34m#  \u{1b}[1;31mA♥ \u{1b}[1;31m3♥ \u{1b}[1;30m10♣ \u{1b}[1;34m#  \u{1b}[1;30mA♣ \u{1b}[1;31m2♥ \u{1b}[1;30m3♣ \u{1b}[1;30m4♣ \u{1b}[1;30m10♣ \u{1b}[1;34m#  \u{1b}[1;30m10♣".to_string(),
                    "12 13 14 15  16 17 18 19 20 21  22 23".to_string()));
    }

    #[test]
    fn contains_joker_1() {
        let cards = Sequence::from_cards(&[
            Joker, 
            RegularCard(Heart, 1),
            RegularCard(Heart, 2),
            RegularCard(Heart, 3),
            RegularCard(Club, 11)
        ]);
        
        assert_eq!(true, cards.contains_joker());
    }
    
    #[test]
    fn contains_joker_2() {
        let cards = Sequence::from_cards(&[
            RegularCard(Heart, 1),
            RegularCard(Heart, 2),
            RegularCard(Heart, 3),
            RegularCard(Club, 11)
        ]);
        
        assert_eq!(false, cards.contains_joker());
    }
    
    #[test]
    fn contains_joker_3() {
        let cards = Sequence::from_cards(&[
            RegularCard(Heart, 1),
            Joker, 
            RegularCard(Heart, 3),
        ]);
        
        assert_eq!(true, cards.contains_joker());
    }
    
    #[test]
    fn card_to_byte_1() {
        let card = Joker;
        let card_byte = card.to_byte();
        assert_eq!(0, card_byte);
    }
    
    #[test]
    fn card_to_byte_2() {
        let card = RegularCard(Heart, 1);
        let card_byte = card.to_byte();
        assert_eq!(1, card_byte);
    }
    
    #[test]
    fn card_to_byte_3() {
        let card = RegularCard(Diamond, 7);
        let card_byte = card.to_byte();
        assert_eq!(33, card_byte);
    }
    
    #[test]
    fn card_to_byte_4() {
        let card = RegularCard(Club, 9);
        let card_byte = card.to_byte();
        assert_eq!(22, card_byte);
    }
    
    #[test]
    fn card_to_byte_5() {
        let card = RegularCard(Spade, 12);
        let card_byte = card.to_byte();
        assert_eq!(51, card_byte);
    }
    
    #[test]
    fn card_to_byte_6() {
        let card = RegularCard(Spade, 13);
        let card_byte = card.to_byte();
        assert_eq!(52, card_byte);
    }
    
    #[test]
    fn card_to_byte_7() {
        let card = RegularCard(Spade, 1);
        let card_byte = card.to_byte();
        assert_eq!(40, card_byte);
    }
    
    #[test]
    fn byte_to_card_1() {
        let byte = 0;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(Joker, card);
    }
    
    #[test]
    fn byte_to_card_2() {
        let byte = 1;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Heart, 1), card);
    }
    
    #[test]
    fn byte_to_card_3() {
        let byte = 20;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Club, 7), card);
    }
    
    #[test]
    fn byte_to_card_4() {
        let byte = 35;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Diamond, 9), card);
    }
    
    #[test]
    fn byte_to_card_5() {
        let byte = 51;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Spade, 12), card);
    }
    
    #[test]
    fn byte_to_card_6() {
        let byte = 52;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Spade, 13), card);
    }
    
    #[test]
    fn byte_to_card_7() {
        let byte = 40;
        let card = Card::from_byte(byte).unwrap();
        assert_eq!(RegularCard(Spade, 1), card);
    }
    
    #[test]
    fn merge_1() {
        let mut seq1 = Sequence::new();
        let seq2 = Sequence::new();
        seq1.merge(seq2);
        assert_eq!(Sequence::new(), seq1);
    }
    
    #[test]
    fn merge_2() {
        let mut seq1 = Sequence::from_cards(&[
            RegularCard(Club, 1),
            RegularCard(Spade, 2),
        ]);
        let seq2 = Sequence::new();
        seq1.merge(seq2);
        assert_eq!(
            Sequence::from_cards(&[
                RegularCard(Club, 1),
                RegularCard(Spade, 2),
            ]),
            seq1);
    }
    
    #[test]
    fn merge_3() {
        let mut seq1 = Sequence::new();
        let seq2 = Sequence::from_cards(&[
            RegularCard(Club, 1),
            RegularCard(Spade, 2),
        ]);
        seq1.merge(seq2);
        assert_eq!(
            Sequence::from_cards(&[
                RegularCard(Spade, 2),
                RegularCard(Club, 1),
            ]),
            seq1);
    }
    
    #[test]
    fn merge_4() {
        let mut seq1 = Sequence::from_cards(&[
            RegularCard(Club,1),
            Joker,
        ]);
        let seq2 = Sequence::from_cards(&[
            RegularCard(Club, 1),
            RegularCard(Spade, 2),
        ]);
        seq1.merge(seq2);
        assert_eq!(
            Sequence::from_cards(&[
                RegularCard(Club, 1),
                Joker,
                RegularCard(Spade, 2),
                RegularCard(Club, 1),
            ]),
            seq1);
    }

    #[test]
    fn no_k_a_jocker_1() {
        let mut seq = Sequence::from_cards(&[
            RegularCard(Club, 13),
            RegularCard(Club, 1),
            Joker
        ]);
        seq.is_valid_sequence_same_suit();
        let exp_seq = Sequence::from_cards(&[
            Joker,
            RegularCard(Club, 13),
            RegularCard(Club, 1)
        ]);
        assert_eq!(seq, exp_seq);
    }
}
