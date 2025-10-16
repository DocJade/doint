// Deck of cards, and manipulation.
// ...the good kind i promise.

// To keep things fair, we dont allow peeking into the deck, you can only draw cards.
// additionally, to prevent cheating on the house side, we cannot see any information about a card
// until it has been flipped over.
// For some games (notably blackjack) the dealer needs to check some information about a card, thus there
// are special methods for those occurrences.

use log::error;

/// A deck of playing cards.
///
/// To keep things fair, you cannot see the inside of the deck
pub struct CardDeck {
    cards: Vec<PlayingCard>,
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
    pub fn shuffle(&mut self) {}

    /// Combine multiple decks of cards into a single deck.
    ///
    /// Does not check for duplicate cards, this simply adds them together.
    ///
    /// This does not shuffle the decks.
    #[must_use]
    pub fn combine(decks: Vec<CardDeck>) -> CardDeck {
        CardDeck {
            cards: decks.into_iter().flat_map(|deck| deck.cards).collect(),
        }
    }

    /// Draw a card from the top of the deck.
    ///
    /// Returns [None] if deck is empty.
    #[must_use]
    pub fn draw(&mut self) -> Option<PlayingCard> {
        // The top of the deck is the end of the vec.
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
                });
            }
        }

        // Only add the jokers if needed.
        if !discard_joker {
            cards.push(PlayingCard {
                suit: CardSuit::Joker,
                rank: CardRank::Joker,
                flipped: false,
            });
            cards.push(PlayingCard {
                suit: CardSuit::Joker,
                rank: CardRank::Joker,
                flipped: false,
            });
        }

        // All done.
        CardDeck { cards }
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
