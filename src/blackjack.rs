use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    pub fn blackjack_value(&self) -> u8 {
        match self.rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a standard 52‑card deck
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);

        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        for suit in suits {
            for rank in ranks {
                cards.push(Card::new(suit, rank));
            }
        }

        Self { cards }
    }

    /// Shuffles the deck
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// Creates a fresh shuffled deck (used each hand so card counting is impossible)
    pub fn fresh_shuffled() -> Self {
        let mut deck = Self::new();
        deck.shuffle();
        deck
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

#[derive(Debug, Clone)]
pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Computes the best blackjack value for the hand
    pub fn value(&self) -> u8 {
        let mut total = 0;
        let mut aces = 0;

        for card in &self.cards {
            let v = card.blackjack_value();
            total += v;
            if card.rank == Rank::Ace {
                aces += 1;
            }
        }

        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        total
    }

    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value() == 21
    }

    pub fn is_bust(&self) -> bool {
        self.value() > 21
    }
}

#[derive(Debug)]
pub struct BlackjackGame {
    pub deck: Deck,
    pub player: Hand,
    pub dealer: Hand,
}

impl BlackjackGame {
    /// Creates a new game with a freshly shuffled deck (reshuffled every hand)
    pub fn new() -> Self {
        let mut deck = Deck::fresh_shuffled();

        let mut player = Hand::new();
        let mut dealer = Hand::new();

        player.add(deck.deal().unwrap());
        dealer.add(deck.deal().unwrap());
        player.add(deck.deal().unwrap());
        dealer.add(deck.deal().unwrap());

        Self { deck, player, dealer }
    }

    pub fn player_hit(&mut self) -> Option<Card> {
        let card = self.deck.deal()?;
        self.player.add(card);
        Some(card)
    }

    pub fn dealer_play(&mut self) {
        while self.dealer.value() < 17 {
            if let Some(card) = self.deck.deal() {
                self.dealer.add(card);
            } else {
                break;
            }
        }
    }

    pub fn result(&self) -> GameResult {
        let player_val = self.player.value();
        let dealer_val = self.dealer.value();

        if self.player.is_bust() {
            return GameResult::DealerWin;
        }

        if self.dealer.is_bust() {
            return GameResult::PlayerWin;
        }

        match player_val.cmp(&dealer_val) {
            std::cmp::Ordering::Greater => GameResult::PlayerWin,
            std::cmp::Ordering::Less => GameResult::DealerWin,
            std::cmp::Ordering::Equal => GameResult::Push,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    PlayerWin,
    DealerWin,
    Push,
}
