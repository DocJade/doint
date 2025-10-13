// hell yeah

// Slots have an expected value that is favored towards the house / bank, thus
// taxes are not collected.

// I like how lazy static works better.
#![allow(clippy::non_std_lazy_statics)]

use crate::models::queries::Users;
use crate::formatting::format_struct::FormattingHelper;
use crate::guards;
use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};
use diesel::Connection;
use lazy_static::lazy_static;
use log::{debug, warn};
use poise::CreateReply;
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateInteractionResponseFollowup,
};
use rand::{rng, seq::IndexedRandom};
use std::iter::repeat_n;
use std::time::Duration;

use crate::models::BankInterface;
use crate::models::bank::transfer::{
    DointTransfer, DointTransferError, DointTransferParty, DointTransferReason,
};
use crate::types::serenity_types::{Context, Error};

use crate::knob::emoji::{
    EMOJI_ANIMATED_ULTRA_FLUSH, EMOJI_BLUNDER, EMOJI_BOOK, EMOJI_BRILLIANT, EMOJI_FERRIS_PARTY,
    EMOJI_FREAKY_CANNY, EMOJI_TRUE, EMOJI_UNCANNY,
};

/// Slot machines
struct SlotMachine<'a> {
    /// The display name of this slot machine
    machine_name: &'a str,

    /// See `SlotPayoutTable`.
    payout_table: SlotPayoutTable,

    /// The set bet amount (in full doints)
    bet_size: BigDecimal,

    /// The layout of the reels on this slot.
    ///
    /// There are always 3 reels.
    ///
    /// All 3 reels are always the same.
    reel_layout: Vec<SlotSymbol>,

    /// The max possible payout. This should be the same as the jackpot, but is here
    /// for convenience.
    ///
    /// `bet_size` * jackpot
    max_possible_payout: BigDecimal,
}

/// It is assumed that any un-covered combination has no payout.
///
/// The payout number is a multiplier on bet size.
struct SlotPayoutTable {
    jackpot: u16,
    triple_red_seven: u16,
    triple_triple_bar: u16,
    triple_double_bar: u16,
    triple_single_bar: u16,
    any_three_bars: u16, // ie b1 b2 b2.
    triple_cherry: u16,
    double_cherry: u16,
    single_cherry: u16,
}

/// Which symbol is this? And what emoji is it?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotSymbol {
    Jackpot(u64),
    RedSeven(u64),
    TripleBar(u64),
    DoubleBar(u64),
    SingleBar(u64),
    Cherry(u64),
    Blank(u64),
}

impl SlotSymbol {
    #[allow(clippy::pedantic)]
    fn get_emoji_id(&self) -> u64 {
        match self {
            SlotSymbol::Jackpot(emoji_id) => *emoji_id,
            SlotSymbol::RedSeven(emoji_id) => *emoji_id,
            SlotSymbol::TripleBar(emoji_id) => *emoji_id,
            SlotSymbol::DoubleBar(emoji_id) => *emoji_id,
            SlotSymbol::SingleBar(emoji_id) => *emoji_id,
            SlotSymbol::Cherry(emoji_id) => *emoji_id,
            SlotSymbol::Blank(emoji_id) => *emoji_id,
        }
    }
}

struct SlotSpinResult {
    /// How much the user won, if anything
    win_amount: BigDecimal,
    /// The result on the slot machine reels.
    reel_result: [SlotSymbol; 3],
    /// Was this the jackpot?
    was_jackpot: bool,
}

//
// Define some slot machines
//

// Slot machine is called "Who wants to be a DILLIONARE?"

// Slot are based off of "Easy Vegas"
// https://easy.vegas/games/slots/par-sheets#freeCasino
// “Easy Vegas” - 97% return to player - (Hit Frequency: 21% - Jackpot Odds: 1 in 9,709)

// Reels
// Reel 1:
// 6 Jackpot
// 8 Red 7
// 9 Triple bar
// 11 Double bar
// 22 Single bar
// 8 Cherry
// 64 Blank (Uncanny)
// Reels 2/3 are the same.
//
// JP = Jackpot         (FreakyCanny)
// R7 = Red 7           (FerrisParty)
// 3B = Triple bar      (Brilliant)
// 2B = Double bar      (Book)
// 1B = Single bar      (Blunder)
// CH = Cherry          (True)
// BL = Blank           (Uncanny)

// Pay table
//  Pay table 3 (97%)
//  Combo               Pays
//  JP JP JP	        1200
//  R7 R7 R7	        150
//  3B 3B 3B	        120
//  2B 2B 2B	        80
//  1B 1B 1B	        40
//  CH CH CH	        10
//  any 3 of 1/2/3B     10
//  any 2 CH	        5
//  any 1 CH	        1

// Lazy static due to the vec in reels

lazy_static! {
    static ref DILLIONARE: SlotMachine<'static> = SlotMachine {
        machine_name: "Who wants to be a Dillionare?!",
        payout_table: SlotPayoutTable {
            jackpot: 1200,
            triple_red_seven: 150,
            triple_triple_bar: 120,
            triple_double_bar: 80,
            triple_single_bar: 40,
            triple_cherry: 10,
            any_three_bars: 10,
            double_cherry: 5,
            single_cherry: 1,
        },
        bet_size: BigDecimal::one(), // One doint, max payout of 1200
        reel_layout: {
            let mut reel: Vec<SlotSymbol> = Vec::new();
            reel.extend(repeat_n(SlotSymbol::Jackpot(EMOJI_FREAKY_CANNY), 6));
            reel.extend(repeat_n(SlotSymbol::RedSeven(EMOJI_FERRIS_PARTY), 8));
            reel.extend(repeat_n(SlotSymbol::TripleBar(EMOJI_BRILLIANT), 9));
            reel.extend(repeat_n(SlotSymbol::DoubleBar(EMOJI_BOOK), 11));
            reel.extend(repeat_n(SlotSymbol::SingleBar(EMOJI_BLUNDER), 22));
            reel.extend(repeat_n(SlotSymbol::Cherry(EMOJI_TRUE), 8));
            reel.extend(repeat_n(SlotSymbol::Blank(EMOJI_UNCANNY), 64));
            reel
        },
        max_possible_payout: BigDecimal::from_usize(1200).expect("1200 should fit."),
    };
}

/// Calculate the payout of a slot machine state
impl SlotMachine<'_> {
    /// Spin the slot machine. Returns a spin result.
    fn spin(&self) -> SlotSpinResult {
        // Calculate the 3 wheels
        let mut rng = rng();
        let wheel1_result: SlotSymbol = *self
            .reel_layout
            .choose(&mut rng)
            .expect("Reel layout should not be empty");
        let wheel2_result: SlotSymbol = *self
            .reel_layout
            .choose(&mut rng)
            .expect("Reel layout should not be empty");
        let wheel3_result: SlotSymbol = *self
            .reel_layout
            .choose(&mut rng)
            .expect("Reel layout should not be empty");

        // Calculate winnings
        let reel_result: [SlotSymbol; 3] = [wheel1_result, wheel2_result, wheel3_result];
        let win_multiplier = calculate_winnings(reel_result, &self.payout_table);

        // Check if its a jackpot
        let was_jackpot = if let Some(amount) = win_multiplier {
            amount == self.payout_table.jackpot
        } else {
            false
        };

        // If we won some amount, multiply it by the win multiplier.
        let win_amount: BigDecimal = if let Some(mult) = win_multiplier {
            &self.bet_size * mult
        } else {
            // No multiplier, no win.
            BigDecimal::zero()
        };

        // All done
        SlotSpinResult {
            win_amount,
            reel_result,
            was_jackpot,
        }
    }
}

// Helper functions
/// Check if all 3 symbols are the same
fn all_same(symbols: [SlotSymbol; 3]) -> bool {
    let first = &symbols[0];
    symbols.iter().all(|this_one| this_one == first)
}

/// Count how many times this symbol occurs
fn occurrences(symbols: [SlotSymbol; 3], to_match: SlotSymbol) -> usize {
    symbols
        .iter()
        .filter(|i| {
            matches!(
                (**i, to_match),
                (SlotSymbol::Cherry(_), SlotSymbol::Cherry(_))
                    | (SlotSymbol::Jackpot(_), SlotSymbol::Jackpot(_))
                    | (SlotSymbol::RedSeven(_), SlotSymbol::RedSeven(_))
                    | (SlotSymbol::TripleBar(_), SlotSymbol::TripleBar(_))
                    | (SlotSymbol::DoubleBar(_), SlotSymbol::DoubleBar(_))
                    | (SlotSymbol::SingleBar(_), SlotSymbol::SingleBar(_))
                    | (SlotSymbol::Blank(_), SlotSymbol::Blank(_))
            )
        })
        .count()
}

/// Check if the symbols only contain bars
fn only_bars(symbols: [SlotSymbol; 3]) -> bool {
    symbols
        .iter()
        .filter(|i| {
            matches!(
                i,
                SlotSymbol::SingleBar(_) | SlotSymbol::DoubleBar(_) | SlotSymbol::TripleBar(_)
            )
        })
        .count()
        == 3
}

/// Calculate winnings of a spin as a multiplier.
///
/// Returns none if nothing was won
fn calculate_winnings(symbols: [SlotSymbol; 3], payouts: &SlotPayoutTable) -> Option<u16> {
    // Are all 3 the same?
    if all_same(symbols) {
        // Which one?
        match symbols[0] {
            SlotSymbol::Jackpot(_) => {
                // User won the jackpot!
                debug!("User won the jackpot!");
                return Some(payouts.jackpot);
            }
            SlotSymbol::RedSeven(_) => {
                return Some(payouts.triple_red_seven);
            }
            SlotSymbol::TripleBar(_) => return Some(payouts.triple_triple_bar),
            SlotSymbol::DoubleBar(_) => return Some(payouts.triple_double_bar),
            SlotSymbol::SingleBar(_) => return Some(payouts.triple_single_bar),
            SlotSymbol::Cherry(_) => return Some(payouts.triple_cherry),
            SlotSymbol::Blank(_) => {
                // Whoop.
                return None;
            }
        }
    }

    // 3 bars?
    if only_bars(symbols) {
        return Some(payouts.any_three_bars);
    }

    // 1 or two cherries?
    let cherries = occurrences(symbols, SlotSymbol::Cherry(1));
    if cherries == 2 {
        return Some(payouts.double_cherry);
    } else if cherries == 1 {
        return Some(payouts.single_cherry);
    }

    // No matches
    None
}

//
// The actual slash command
//

/// Play slots!
#[allow(clippy::too_many_lines)] // TODO: yeah
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = 5,
    check = guards::in_doints_category,
    check = guards::ctx_member_enrolled_in_doints,
    check = guards::in_casino
)]
pub(crate) async fn slots(
    ctx: Context<'_>,
    // #[description = "Which machine would you like to play?"] // TODO: more slot machines
    // machine: Coin,
) -> Result<(), Error> {
    // One day we'll support multiple slots, hence the var here
    let machine = &DILLIONARE;

    // Get the database pool
    let pool = ctx.data().db_pool.clone();

    // If we run again, we want to update the original message.
    let mut looped: Option<poise::ReplyHandle<'_>> = None;

    // Run in a loop, in case user wants to spin again.
    loop {
        // Get a connection
        let mut conn = pool.get()?;

        // Get the user that is betting
        let Some(better) = Users::get_doint_user(ctx.author().id, &mut conn)? else {
            // Has role, but not in DB.
            // TODO: error for this / correction
            let _ = ctx
                .say("Uhh, you're not in the doint DB properly, tell doc.")
                .await?;
            return Ok(());
        };

        // Make sure the user can afford the bet.
        let required_doints: &BigDecimal = &machine.bet_size;
        if &better.bal < required_doints {
            // User cant afford bet.
            let bet_string = FormattingHelper::display_doint(required_doints);
            let _ = ctx
                .say(format!("You cannot afford the {bet_string} bet."))
                .await?;
            return Ok(());
        }

        // Make sure the bank can afford the jackpot.
        let bank_bal: &BigDecimal = &BankInterface::get_bank_balance(&mut conn)?;
        let max_payout: &BigDecimal = &machine.max_possible_payout;
        if bank_bal < max_payout {
            // Bank cant pay that out.
            warn!("Bank cant afford slots!");
            let _ = ctx.say("The bank currently doesn't have enough money for that slot machine. Try again later.".to_string()).await?;
            return Ok(());
        }

        // User can afford it. Do the spin.
        // We immediately know the outcome, but we want that animation.

        // Run the slot
        let spin_result = machine.spin();
        let guild_id = ctx.guild_id().expect("Has to run in doccord.");

        // Get the emoji we need.
        let roller = guild_id
            .emoji(ctx, EMOJI_ANIMATED_ULTRA_FLUSH.into())
            .await?
            .to_string();

        let one_id = spin_result.reel_result[0].get_emoji_id();
        let one_emoji = guild_id.emoji(ctx, one_id.into()).await?.to_string();
        let two_id = spin_result.reel_result[1].get_emoji_id();
        let two_emoji = guild_id.emoji(ctx, two_id.into()).await?.to_string();
        let three_id = spin_result.reel_result[2].get_emoji_id();
        let three_emoji = guild_id.emoji(ctx, three_id.into()).await?.to_string();

        // Text for the outcome
        let amount_actually_won: BigDecimal = &spin_result.win_amount * &machine.bet_size;

        let result_text: String = if spin_result.win_amount > BigDecimal::zero() {
            // User won some.
            // Jackpot text if they won that too
            let jackpot_text = if spin_result.was_jackpot {
                "# YOU HIT THE JACKPOT!\n"
            } else {
                // nothing
                ""
            };

            let formatted_doints = FormattingHelper::display_doint(&amount_actually_won);

            format!("{jackpot_text}You won {formatted_doints}!")
        } else {
            // User lost.
            "Too bad.".to_string()
        };

        // We actually pay the user before displaying anything, in-case that fails.

        conn.transaction::<(), DointTransferError, _>(|conn| {
            // If user broke even, we dont need to do anything at all.
            if amount_actually_won == machine.bet_size {
                // Broke even, no action.
                return Ok(());
            }

            // Take the user's bet money
            let transfer = DointTransfer {
                sender: DointTransferParty::DointUser(ctx.author().id.get()),
                recipient: DointTransferParty::Bank,
                transfer_amount: machine.bet_size.clone(),
                apply_fees: false, // Slots aren't taxed.
                transfer_reason: DointTransferReason::CasinoLoss,
            };
            BankInterface::bank_transfer(conn, transfer)?;

            // Now give them their winnings, if needed
            if spin_result.win_amount == BigDecimal::zero() {
                // User lost, nothing left to do
                return Ok(());
            }

            // User won something!
            let transfer = DointTransfer {
                sender: DointTransferParty::Bank,
                recipient: DointTransferParty::DointUser(ctx.author().id.get()),
                transfer_amount: amount_actually_won,
                apply_fees: false,
                transfer_reason: DointTransferReason::CasinoWin,
            };

            BankInterface::bank_transfer(conn, transfer)?;
            Ok(())
        })?;

        // Money has been transferred, now we can display things

        let template = |machine_name: &str,
                        slot_1_emoji: &String,
                        slot_2_emoji: &String,
                        slot_3_emoji: &String,
                        result_message: &String| {
            format!(
                "***{machine_name}***\n{slot_1_emoji}{slot_2_emoji}{slot_3_emoji}\n-# {result_message}"
            )
        };

        // Spacer to prevent the window from jumping around
        let vertical_spacing: String = "Spinning...".to_string();

        // One
        // Send the initial message if this is the first go-around
        let handle: poise::ReplyHandle<'_> = if let Some(old) = looped.take() {
            // Put in the roller.
            old.edit(
                ctx,
                CreateReply::default().content(template(
                    machine.machine_name,
                    &roller,
                    &roller,
                    &roller,
                    &vertical_spacing,
                )),
            )
            .await?;
            std::thread::sleep(Duration::from_secs_f64(rand::random_range(0.0..1.0)));
            old
        } else {
            // First run.
            let handle = ctx
                .say(template(
                    machine.machine_name,
                    &roller,
                    &roller,
                    &roller,
                    &vertical_spacing,
                ))
                .await?;
            // Now wait a bit for dramatic effect
            std::thread::sleep(Duration::from_secs_f64(rand::random_range(0.5..2.0)));
            handle
        };

        // One
        handle
            .edit(
                ctx,
                CreateReply::default().content(template(
                    machine.machine_name,
                    &one_emoji,
                    &roller,
                    &roller,
                    &vertical_spacing,
                )),
            )
            .await?;
        std::thread::sleep(Duration::from_secs_f64(rand::random_range(0.0..0.5)));

        // Two
        handle
            .edit(
                ctx,
                CreateReply::default().content(template(
                    machine.machine_name,
                    &one_emoji,
                    &two_emoji,
                    &roller,
                    &vertical_spacing,
                )),
            )
            .await?;
        std::thread::sleep(Duration::from_secs_f64(rand::random_range(0.0..0.5)));

        // Three
        handle
            .edit(
                ctx,
                CreateReply::default().content(template(
                    machine.machine_name,
                    &one_emoji,
                    &two_emoji,
                    &three_emoji,
                    &vertical_spacing,
                )),
            )
            .await?;
        std::thread::sleep(Duration::from_secs_f64(rand::random_range(0.0..0.5)));

        // Result.
        handle
            .edit(
                ctx,
                CreateReply::default().content(template(
                    machine.machine_name,
                    &one_emoji,
                    &two_emoji,
                    &three_emoji,
                    &result_text,
                )),
            )
            .await?;

        // Button that asks if they'd like to play again.

        // Need a uuid so we dont re-trigger someone elses spin
        let spin_again_uuid = ctx.id();

        // The color of the button depends on if you won or not.
        let spin_button_style = if spin_result.win_amount > BigDecimal::zero() {
            // Green, positive enforcement.
            ButtonStyle::Success
        } else {
            ButtonStyle::Danger
        };

        let spin_button: CreateButton = CreateButton::new(format!("{spin_again_uuid}"))
            .label("Spin again")
            .style(spin_button_style);
        let action_row: CreateActionRow = CreateActionRow::Buttons(vec![spin_button]);
        handle
            .edit(ctx, CreateReply::default().components(vec![action_row]))
            .await?;

        // User has 10 seconds to spin again.
        let mut spin_again: bool = false;
        while let Some(interaction) = ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(10))
            .filter(move |mci| mci.data.custom_id == spin_again_uuid.to_string())
            .await
        {
            interaction.defer(ctx).await?; // Should fix interaction failed issues.
            // Make sure it was the same user
            if interaction
                .clone()
                .member
                .expect("Slots in doccord")
                .user
                .id
                != ctx.author().id
            {
                // Not the right user.
                interaction
                    .create_followup(
                        ctx,
                        CreateInteractionResponseFollowup::new()
                            .ephemeral(true)
                            .content("Get your own machine, asshole!"),
                    )
                    .await?;
                continue;
            }
            // User pressed the button again, SPIN!
            spin_again = true;
            break;
        }

        if spin_again {
            // Grey out the spin again button
            let cant_spin: CreateButton = CreateButton::new(format!("{spin_again_uuid}"))
                .label("Spin again")
                .disabled(true);
            let cant_spin_action_row: CreateActionRow = CreateActionRow::Buttons(vec![cant_spin]);
            handle
                .edit(
                    ctx,
                    CreateReply::default().components(vec![cant_spin_action_row]),
                )
                .await?;

            // Again!
            looped = Some(handle);
            continue;
        }

        // Button was not pushed, remove it
        handle
            .edit(ctx, CreateReply::default().components(vec![]))
            .await?;

        // All done.
        break;
    }
    Ok(())
}
