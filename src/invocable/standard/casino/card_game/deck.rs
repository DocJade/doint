// Deck of cards, and manipulation.
// ...the good kind i promise.

// To keep things fair, we dont allow peeking into the deck, you can only draw cards.
// additionally, to prevent cheating on the house side, we cannot see any information about a card
// until it has been flipped over.
// For some games (notably blackjack) the dealer needs to check some information about a card, thus there
// are special methods for those occurrences.

use log::error;
use rand::seq::SliceRandom;

/// A deck of playing cards.
///
/// To keep things fair, you cannot see the inside of the deck
pub struct CardDeck {
    /// Cards that have not been played yet.
    cards: Vec<PlayingCard>,
    /// Cards that have been discarded. These will be shuffled back into the deck
    /// automatically when the normal draw pile runs out of cards.
    ///
    /// Due to not being able to `Drop` cards, this will contain every card that has been in play,
    /// but is not currently being used.
    discarded_cards: Vec<PlayingCard>,
}

/// A single playing card.
///
/// Use methods on the card to get information about it. Lack of direct
/// access prevents tampering.
///
/// You cannot get information about a card until you flip it.
pub struct PlayingCard {
    suit: CardSuit,
    rank: CardRank,
    /// Has this card been revealed?
    flipped: bool,

    /// This is a handle to prevent the card from being dropped automatically.
    /// Cards must be returned to the deck.
    #[allow(dead_code)]
    handle: Undroppable<bool>,
}

/// The suit of a playing card.
///
/// Joker has its own suit, due to it not being in any suit.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CardSuit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    Joker,
}

/// The rank of a playing card.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CardRank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

/// The color of a card.
///
/// Jokers don't have a set color.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CardColor {
    Red,
    Black,
    Joker,
}

impl CardDeck {
    /// Shuffles (or re-shuffles) all of the cards currently in the deck.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Combine multiple decks of cards into a single deck.
    ///
    /// Does not check for duplicate cards, this simply adds them together.
    ///
    /// This does not shuffle the decks.
    #[must_use]
    pub fn combine(decks: Vec<CardDeck>) -> CardDeck {
        // Use fold to combine multiple decks into a single deck
        // Starting with empty vectors for both cards and discarded cards
        let (cards, discarded_cards) = decks.into_iter().fold(
            (Vec::new(), Vec::new()), // (cards, discarded)
            |(mut cards, mut discarded), deck| {
                // For each deck
                // Add all cards from this deck to our combined cards
                cards.extend(deck.cards);
                // Add all discarded cards from this deck to our combined discarded pile
                discarded.extend(deck.discarded_cards);
                // Return the updated vectors
                (cards, discarded)
            },
        );

        // Create new deck with all combined cards
        CardDeck {
            cards,
            discarded_cards,
        }
    }

    /// Draw a card from the top of the deck.
    ///
    /// Shuffles back in the discard pile if deck is out of cards.
    ///
    /// Returns [None] if deck, is empty, and there were no cards in the discard pile.
    #[must_use]
    pub fn draw(&mut self) -> Option<PlayingCard> {
        // The top of the deck is the end of the vec.
        // If we're out of cards, shuffle in the discard deck.
        if self.cards.is_empty() {
            self.cards.append(&mut self.discarded_cards);
            self.shuffle();
        }

        // Get a card.
        self.cards.pop()
    }

    /// Creates a new deck of cards, optionally removing the joker.
    #[must_use]
    pub fn new(discard_joker: bool) -> CardDeck {
        // TIL that a deck has 2 jokers, i thought it was just one.

        let mut cards = Vec::with_capacity(if discard_joker { 52 } else { 54 });

        // Nested for loops, yay
        for suit in [
            CardSuit::Clubs,
            CardSuit::Diamonds,
            CardSuit::Hearts,
            CardSuit::Spades,
        ] {
            for rank in [
                CardRank::Two,
                CardRank::Three,
                CardRank::Four,
                CardRank::Five,
                CardRank::Six,
                CardRank::Seven,
                CardRank::Eight,
                CardRank::Nine,
                CardRank::Ten,
                CardRank::Jack,
                CardRank::Queen,
                CardRank::King,
                CardRank::Ace,
            ] {
                cards.push(PlayingCard {
                    suit,
                    rank,
                    flipped: false, // All cards in a deck start face down.
                    handle: Undroppable::new_unchecked(true),
                });
            }
        }

        // Only add the jokers if needed.
        if !discard_joker {
            cards.push(PlayingCard {
                suit: CardSuit::Joker,
                rank: CardRank::Joker,
                flipped: false,
                handle: Undroppable::new_unchecked(true),
            });
            cards.push(PlayingCard {
                suit: CardSuit::Joker,
                rank: CardRank::Joker,
                flipped: false,
                handle: Undroppable::new_unchecked(true),
            });
        }

        // All done.
        CardDeck {
            cards,
            discarded_cards: vec![],
        }
    }
}

impl PlayingCard {
    /// Get the suit of this card.
    ///
    /// Returns [None] if card has not yet been flipped.
    #[must_use]
    pub fn suit(&self) -> Option<CardSuit> {
        if !self.flipped {
            return None;
        }
        Some(self.suit)
    }

    /// Get the rank of this card.
    ///
    /// Returns [None] if card has not yet been flipped.
    #[must_use]
    pub fn rank(&self) -> Option<CardRank> {
        if !self.flipped {
            return None;
        }
        Some(self.rank)
    }

    /// Get the color of this card.
    ///
    /// Returns [None] if card has not yet been flipped.
    #[must_use]
    pub fn color(&self) -> Option<CardColor> {
        if !self.flipped {
            return None;
        }

        // Jokers have no color.
        if self.rank == CardRank::Joker {
            return Some(CardColor::Joker);
        }

        // In a standard deck, clubs and spades are black.
        if matches!(self.suit, CardSuit::Clubs | CardSuit::Spades) {
            Some(CardColor::Black)
        } else {
            // Must be hearts or diamonds.
            Some(CardColor::Red)
        }
    }

    /// Flips the card over. You cannot un-flip a card, thus subsequent calls to this method do nothing.
    pub fn flip(&mut self) {
        self.flipped = true;
    }

    /// Special case for blackjack, dealer may need to check for an ace.
    ///
    /// This requires an input card (The key card) that necessitates the check. IE a 10 value card (10, Jack, Queen, King)
    ///
    /// Returns true if the hole card is an ace.
    /// Returns [None] if an invalid input card was provided.
    #[must_use]
    pub fn blackjack_ace_peek(key_card: &PlayingCard, hole_card: &PlayingCard) -> Option<bool> {
        // The key card must be flipped.
        if !key_card.flipped {
            return None;
        }

        // The key card must have a value of 10.
        match key_card.rank {
            CardRank::Ten | CardRank::Jack | CardRank::Queen | CardRank::King => {}
            CardRank::Joker => {
                // The pit boss is about to kill the dealer.
                error!("Playing blackjack with a Joker!");
                // Just kill the thread with this hand lmao.
                unreachable!("Blackjack should not have jokers.")
            }
            _ => {
                // This is not a valid card.
                return None;
            }
        }

        // Key card is valid.
        // Make sure its not the joker.
        if hole_card.rank == CardRank::Joker {
            unreachable!("Blackjack should not have jokers.")
        }

        // Do the ace check.
        Some(hole_card.rank == CardRank::Ace)
    }

    /// Special case for blackjack, dealer may need to check for a 10 ranked card.
    ///
    /// This requires an input card (The key card) that necessitates the check. IE the card must be an ace.
    ///
    /// Returns true if the hole card is 10 ranked.
    /// Returns [None] if an invalid input card was provided.
    #[must_use]
    pub fn blackjack_ten_peek(key_card: &PlayingCard, hole_card: &PlayingCard) -> Option<bool> {
        // The key card must be flipped.
        if !key_card.flipped {
            return None;
        }

        // The key card must be an ace.
        match key_card.rank {
            CardRank::Ace => {}
            CardRank::Joker => {
                // The pit boss is about to kill the dealer.
                error!("Playing blackjack with a Joker!");
                // Just kill the thread with this hand lmao.
                unreachable!("Blackjack should not have jokers.")
            }
            _ => {
                // This is not a valid card.
                return None;
            }
        }

        // Key card is valid.
        // Make sure its not the joker.
        if hole_card.rank == CardRank::Joker {
            unreachable!("Blackjack should not have jokers.")
        }

        // Check for 10 ranked card.
        Some(matches!(
            hole_card.rank,
            CardRank::Ten | CardRank::Jack | CardRank::Queen | CardRank::King
        ))
    }
}

/// How to score cards based on different scoring methods.
pub enum CardScoring {
    /// Standard poker scoring, where aces are high and all face cards are worth their rank.
    Poker,
    /// Blackjack scoring, where aces can be 1 or 11 and face cards are worth 10.
    ///
    /// The boolean denotes wether the card should be counted high or low, this only affects aces.
    ///
    /// True for high, false for low.
    Blackjack(bool),
    /// Standard numeric scoring, where aces are 1 and face cards are 11-13.
    Standard,
}

impl CardRank {
    /// Get the value of this card as an integer.
    ///
    /// Must provide a scoring key.
    #[must_use]
    pub fn value(&self, method: &CardScoring) -> u8 {
        match method {
            CardScoring::Poker => match self {
                CardRank::Two => 2,
                CardRank::Three => 3,
                CardRank::Four => 4,
                CardRank::Five => 5,
                CardRank::Six => 6,
                CardRank::Seven => 7,
                CardRank::Eight => 8,
                CardRank::Nine => 9,
                CardRank::Ten => 10,
                CardRank::Jack => 11,
                CardRank::Queen => 12,
                CardRank::King => 13,
                CardRank::Ace => 14,
                CardRank::Joker => 0,
            },
            CardScoring::Blackjack(high) => match self {
                CardRank::Two => 2,
                CardRank::Three => 3,
                CardRank::Four => 4,
                CardRank::Five => 5,
                CardRank::Six => 6,
                CardRank::Seven => 7,
                CardRank::Eight => 8,
                CardRank::Nine => 9,
                CardRank::Ten | CardRank::Jack | CardRank::Queen | CardRank::King => 10,
                CardRank::Ace => {
                    if *high {
                        11
                    } else {
                        1
                    }
                }
                CardRank::Joker => 0,
            },
            CardScoring::Standard => match self {
                CardRank::Two => 2,
                CardRank::Three => 3,
                CardRank::Four => 4,
                CardRank::Five => 5,
                CardRank::Six => 6,
                CardRank::Seven => 7,
                CardRank::Eight => 8,
                CardRank::Nine => 9,
                CardRank::Ten => 10,
                CardRank::Jack => 11,
                CardRank::Queen => 12,
                CardRank::King => 13,
                CardRank::Ace => 1,
                CardRank::Joker => 0,
            },
        }
    }
}

// Since we need to keep track of all of the playing cards, we need to enforce that they cannot be dropped.
// Thus callers MUST return the cards to the discard pile after they are done using them.
// https://jack.wrenn.fyi/blog/undroppable/

/// A type that cannot be dropped.
pub struct Undroppable<T: ?Sized>(std::mem::ManuallyDrop<T>);

impl<T> Undroppable<T> {
    // Makes `val` undroppable.
    //
    // If `val` has a  non-trivial destructor, attempting
    // to drop it will result in a compilation error.
    pub fn new_unchecked(val: T) -> Self {
        Self(std::mem::ManuallyDrop::new(val))
    }
}

impl<T: ?Sized> Drop for Undroppable<T> {
    fn drop(&mut self) {
        const {
            assert!(!std::mem::needs_drop::<T>(), "This cannot be dropped.");
        }
    }
}
