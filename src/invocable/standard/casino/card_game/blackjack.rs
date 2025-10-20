// Counting cards isn't possible 'round here

use bigdecimal::{BigDecimal, FromPrimitive};

use crate::{
    invocable::standard::casino::card_game::deck::{CardDeck, CardRank, PlayingCard},
    prelude::*,
};

// House edge is due to 8 decks, hit on soft 17.
// This edge is _tiny_, thus additional edge comes from taxing winnings.
// https://wizardofodds.com/games/blackjack/expected-values/

// I actually dont know the rules of blackjack that well, so i'm documenting them here.
// rules transcribed from https://bicyclecards.com/how-to-play/blackjack

// Variation rules:
// Played with 8 decks.
// Blackjack pays 3:2
// Splits allowed.
// - Up to 4 splits
// Dealer hits on soft 17.
// No drawing to split aces
// - After splitting 2 aces, player gets a card for each of their new splits, and their turn immediately ends.

// A "soft" 17 is an ace + a ten card.
// Aces count as either 1 or 11.

//
// Blackjack state machine:
//

// == Card Values ==
// 2-10: Face value
// Jack, Queen, King: 10
// Ace: 1 or 11 (whichever is most advantageous without busting)

// == Setup phase ==
// Player places bet.
// - 1 Doint minimum bet, no maximum (as long as bank can afford)
// Dealer gets 8 jokerless decks.
// Dealer combines all decks.
// Dealer shuffles deck.
// Player gets 1 card. (Face up)
// Dealer gets 1 card. (Face up)
// Player gets 1 card. (Face up)
// Dealer gets 1 card. (Face down, this is the hole card.)

// == Natural check ==

// If player has natural blackjack:
// - If dealer also has blackjack:
// - - Tie, player gets bet back.
// - Dealer does NOT have blackjack:
// - - Player wins.

// == Insurance ==
// Only offered when revealed dealer card is an ace
// Costs 50% of bet amount
// Pays 2:1 if dealer has blackjack
// Insurance bet is lost if dealer doesn't have blackjack

// == Player turn ==

// If both cards have same rank, player can split.
// - Split hand must be paid for with the same initial bet.
// - Make sure the bank can afford that.

// == Split Details ==
// Unlimited splitting allowed.
// If a freshly split hand gets dealt a hand that can allow another split, you can split.
// You can double down on a split hand.
// Split aces receive only one card each and can't be hit

// Player can also choose to double down

// == Double Down ==
// Available after receiving initial two cards, always allowed.
// Double original bet
// Receive exactly one more card
// Allowed on any first two cards
// No double after split

// = Hit or Stand.
// Hit:
// - Player gets 1 card. (Face up)
// - If player total is now over 21, bust.
// Stand:
// - It is now the dealers turn

// == Dealer turn ==

// Turn over hole card.
// If dealer has blackjack (and the player doesn't), do insurance if user paid, otherwise dealer wins, or a push.
// If dealer has ACE and counting it as 11 lands in 18..=21:
// - Dealer MUST stand.
// If soft 17:
// - Dealer MUST hit.
// If <=16:
// - Hit
// else:
// - Stand
// If dealer has total >21:
// - Dealer busts, all remaining players win.

// == Payout phase ==

// For each non-busted player hand:
// If player total > dealer total or dealer busted:
// - Player wins 1:1 (or 3:2 for natural blackjack)
// If player total = dealer total:
// - Push (tie), player gets bet back
// If player total < dealer total:
// - Player loses entire bet

/// Play blackjack!
#[poise::command(slash_command, guild_only, check = guards::in_casino)]
pub async fn blackjack(
    ctx: PoiseContext<'_>,
    #[description = "How much are you betting?"]
    #[min = 1] // Must bet at least 1.00 Doint.
    bet: f64, // Bet sizes are not limited, but bank must be able to pay it out.
) -> Result<(), BotError> {
    // Turn that float into a BigDecimal
    let Some(bet) = BigDecimal::from_f64(bet) else {
        // Failed to cast!
        return Err(BotError::BigDecimalCast);
    };

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // Get a connection
    let mut conn = pool.get()?;

    // To run the slash command the player must be in the DB, we wont check again here.
    let player = Users::get_doint_user(ctx.author().id, &mut conn)?
        .expect("Pre-command check should catch user not being in db / enrolled");

    // Make sure the user can afford the bet.
    let bet_size: BigDecimal = if player.bal < bet {
        // Unlike coinflip, we dont auto-increase to max amount.
        // private response as to not embarrass the player
        ctx.defer_ephemeral().await?;
        let _ = ctx.say("You can't afford that bet.").await?;
        return Ok(());
    } else {
        bet.clone()
    };

    // We don't actually check if the bank can afford the bet here, we will continue to pay out the user until the
    // bank runs out of money, unless the bank couldn't afford a single 3:2 payment.
    if BankInterface::get_bank_balance(&mut conn)?
        < bet_size * BigDecimal::from_f64(1.5).expect("1.5 fits.")
    {
        // bank couldn't pay this bet out
        let _ = ctx
            .say("The bank doesn't have enough money for that bet, sorry.")
            .await?;
        return Ok(());
    }

    // == Setup phase ==

    // Get the 4 decks.
    // Fun trick with mapping iterators!
    let decks: Vec<CardDeck> = (0..4).map(|_| CardDeck::new(true)).collect();

    // Put them all together
    let mut deck: CardDeck = CardDeck::combine(decks);

    // The single shuffle.
    deck.shuffle();

    // Deal initial cards

    let mut player_hand: BlackJackHand = BlackJackHand::new(false);
    let mut dealer_hand: BlackJackHand = BlackJackHand::new(true);

    player_hand.deal(&mut deck, false);
    dealer_hand.deal(&mut deck, false);
    player_hand.deal(&mut deck, false);
    dealer_hand.deal(&mut deck, true); // Face down, hole card.

    // == Natural check ==

    // Check for natural blackjack
    if player_hand.is_blackjack() {
        // Nice! See if dealer has blackjack.
        dealer_hand.cards[1].flip();
        if dealer_hand.is_blackjack() {
            // Tie! Player gets bet back
            todo!()
        } else {
            // User wins!
            todo!()
        }
    }

    // == Insurance ==

    // Offer insurance if needed.
    if dealer_hand.possible_blackjack() {
        // Enable insurance button
        todo!()
    }

    // == Player turn ==
    let mut hands_to_play: Vec<BlackJackHand> = vec![player_hand];
    let mut finished_hands: Vec<BlackJackHand> = vec![];

    while let Some(hand) = hands_to_play.pop() {
        // Splitting is applicable.
        if hand.can_split() {
            // Ask user if they want to split, if they split, put that hand off to the side and deal them another card.
            // Keep splitting until user either declines split or they cannot split.

            todo!()
            // push split hands to hands to play
        }

        // Doubling down
        if hand.cards.len() == 2 && !hand.from_split {
            // Ask player if they want to double down.
            todo!()
            // Doubling down finishes this hand.
        }

        // Let player hit
        while !hand.is_busted() {
            // Ask if player wants to hit or stand.
            todo!()
            // if stand { break }
        }

        // Done with this hand.
        finished_hands.push(hand);
    }

    // == Dealer turn ==

    // Flip hole card.
    dealer_hand.cards[1].flip();

    // Check for blackjack
    if dealer_hand.is_blackjack() {
        // Skip everything.
        todo!()
    }

    // Check hand to see if it lands in 18..21 when scored high.
    if (18..21).contains(&dealer_hand.high_score()) {
        // dealer must stand.
        todo!()
    }

    // Soft 17?
    if dealer_hand.low_score() == 17 {
        // Dealer must hit.
        todo!()
    }

    // Hit if <=16
    if dealer_hand.auto_score().0 <= 16 {
        // hit!
        todo!()
    } else {
        // must stand.
    }

    // == Payout phase ==

    // Did the dealer bust?
    let dealer_busted: bool = dealer_hand.is_busted();
    let dealer_score: u8 = dealer_hand.auto_score().0;

    // Loop over all of the non-busted hands.
    for hand in finished_hands.iter().filter(|h| !h.is_busted()) {
        let hand_score = hand.auto_score().0;
        // If player has more than the dealer, or the dealer busted, they win
        if (hand_score > dealer_score) || dealer_busted {
            // Player wins!
            // Pay rate changes if natural blackjack.
            if hand.is_blackjack() {
                // 3:2 !
                todo!()
            } else {
                // 1:1
                todo!()
            }
        }

        // tie?
        if hand_score == dealer_score {
            // Push, no money moves.
            todo!()
        }

        // Player lost
        // If dealer had blackjack, check for insurance
        if dealer_hand.is_blackjack() && hand.took_insurance {
            // Player took insurance. Since insurance is 50% of the bet and pays 2:1, the
            // user broke even on this entire hand. Push.
            todo!()
        }

        // Take money for this hand.
        todo!()
    }

    todo!()
}

/// A blackjack hand (as in the cards, not the round)
#[allow(clippy::struct_excessive_bools)] // Really? 3 bool limit? lmao
struct BlackJackHand {
    /// The cards within the hand.
    cards: Vec<PlayingCard>,
    /// Is this a dealer's hand?
    dealer: bool,
    /// Has this hand been doubled down?
    doubled_down: bool,
    /// Was this hand split off from another hand?
    from_split: bool,
    /// Did the player take insurance on this hand?
    took_insurance: bool,
}

impl BlackJackHand {
    /// Score a hand. Will return the score of the hand, and whether the hand is soft or hard.
    ///
    /// Does not score unrevealed cards, they are counted as 0.
    ///
    /// Returns (score, `is_soft`)
    #[must_use]
    fn auto_score(&self) -> (u8, bool) {
        let (low, high) = self.score_hand();

        // if the high score (counting aces as 11) is less than or equal to 21
        if high <= 21 {
            // return the high score and whether the hand is "soft" (has an ace counted as 11)
            (high, high != low)
        } else {
            // if high score busts, return the low score (counting all aces as 1)
            // hand is "hard" since all aces must be counted as 1 to avoid busting
            (low, false)
        }
    }

    /// Get the high score of a hand (11 ace).
    #[must_use]
    fn high_score(&self) -> u8 {
        self.score_hand().0
    }

    /// Get the low score of a hand (1 ace).
    #[must_use]
    fn low_score(&self) -> u8 {
        self.score_hand().1
    }

    /// Get both the high and low score of a hand.
    ///
    /// Returns (low, high)
    #[must_use]
    fn score_hand(&self) -> (u8, u8) {
        self.cards
            .iter()
            .filter_map(super::deck::PlayingCard::rank)
            .fold((0u8, 0u8), |(low_acc, high_acc), rank| {
                (
                    low_acc.saturating_add(rank.value(&super::deck::CardScoring::Blackjack(false))),
                    high_acc.saturating_add(rank.value(&super::deck::CardScoring::Blackjack(true))),
                )
            })
    }

    /// Quick check for a blackjack.
    ///
    /// Hand must only have 2 cards
    #[must_use]
    fn is_blackjack(&self) -> bool {
        self.auto_score().0 == 21 && self.cards.len() == 2
    }

    /// Has this hand busted?
    #[must_use]
    fn is_busted(&self) -> bool {
        self.auto_score().0 > 21
    }

    /// Make a new hand
    #[must_use]
    fn new(is_dealer: bool) -> Self {
        Self {
            cards: vec![],
            dealer: is_dealer,
            doubled_down: false,
            from_split: false,
            took_insurance: false,
        }
    }

    /// Split a hand.
    ///
    /// The dealer cannot split, and you cannot split a doubled down hand.
    #[must_use]
    fn split(&mut self) -> Option<[Self; 2]> {
        if self.dealer || self.doubled_down || self.cards.len() != 2 {
            // Cannot split.
            return None;
        }

        let right_hand: Self = Self {
            cards: vec![self.cards.pop().expect("Checked len.")],
            dealer: self.dealer,
            doubled_down: self.doubled_down,
            from_split: true,
            took_insurance: self.took_insurance,
        };
        let left_hand: Self = Self {
            cards: vec![self.cards.pop().expect("Checked len.")],
            dealer: self.dealer,
            doubled_down: self.doubled_down,
            from_split: true,
            took_insurance: self.took_insurance,
        };

        Some([left_hand, right_hand])
    }

    /// Double down a hand.
    ///
    /// Dealer cannot double down, and thus has no effect.
    fn double_down(&mut self) {
        if self.dealer {
            return;
        }
        self.doubled_down = true;
    }

    /// Deal a card into this hand. Optionally face down if this is the dealer.
    fn deal(&mut self, deck: &mut CardDeck, face_down: bool) {
        // Give a card
        // We expect to always get cards due to the massive deck size and automatic re-shuffling of
        // the discard pile.
        self.cards
            .push(deck.draw().expect("We shouldn't run out of cards."));
        if face_down && self.dealer {
            // no need to flip the card.
            return;
        }

        // flip the card over.
        self.cards
            .last_mut()
            .expect("there is at least one card in the deck")
            .flip();
    }

    /// Check if a blackjack is possible by flipping over the hole card.
    ///
    /// Hand must be two cards, and the only shown card (also the first card) is the ace.
    fn possible_blackjack(&self) -> bool {
        // Need 2 cards.
        if self.cards.len() != 2 {
            return false;
        }

        // First card must be an ace, or a value of 10.
        if let Some(rank) = self.cards[0].rank()
            && (rank != CardRank::Ace
                || rank.value(&super::deck::CardScoring::Blackjack(false)) == 10)
        {
            return false;
        }

        // Second card must be face down.
        if self.cards[1].color().is_some() {
            return false;
        }

        true
    }

    /// Check if this hand can be split.
    ///
    /// Hand must have only 2 cards, and they must be the same rank.
    fn can_split(&self) -> bool {
        // Need 2 cards.
        if self.cards.len() != 2 {
            return false;
        }

        // Cannot split a double'd down hand.
        if self.doubled_down {
            return false;
        }

        // Both cards must be showing.
        if self.cards[0].color().is_none() || self.cards[1].color().is_none() {
            return false;
        }

        // Are cards the same?
        self.cards[0].rank().expect("Guarded.") == self.cards[1].rank().expect("Guarded.")
    }
}
