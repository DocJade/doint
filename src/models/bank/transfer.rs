// Move doints from one place to another, wether that be between users or between the bank and elsewhere.

use bigdecimal::{BigDecimal, Zero};
use diesel::{Connection, MysqlConnection};
use log::warn;
use thiserror::Error;

use crate::models::data::bank::BankInfo;
use crate::models::data::users::DointUser;
use crate::schema::bank::dsl::bank;
use crate::{models::BankInterface, models::queries::Users};
use diesel::prelude::*;

/// Struct for facilitating doint transfers between two parties.
///
/// Please read the documentation of the struct fields for requirements.
pub(crate) struct DointTransfer {
    /// Where the doints are coming from.
    pub(crate) sender: DointTransferParty,

    /// Where the doints are going to.
    pub(crate) recipient: DointTransferParty,

    /// The amount of doints being sent.
    ///
    /// This must be a positive number. If you wish to take doints from one place, simply swap
    /// the order of sender and recipient.
    pub(crate) transfer_amount: BigDecimal,

    /// Do fees apply to this transaction?
    ///
    /// For example, you shouldn't be collecting transfer fees while collecting taxes.
    ///
    /// Transfers out of the bank cannot incur transfer fees, that would be pointless.
    pub(crate) apply_fees: bool,

    /// Why this transfer is being made.
    ///
    /// User payments must happen between 2 users, no other combinations are allowed.
    pub(crate) transfer_reason: DointTransferReason,
}

/// Enum for picking where doints are being transferred to.
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum DointTransferParty {
    /// The central Doint bank.
    Bank,
    /// A user. Must provide their discord user ID.
    DointUser(u64),
}

/// Why this transfer is occurring (for logging and such)
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum DointTransferReason {
    TaxCollection,
    CasinoLoss,
    CasinoWin,
    UniversalBasicIncome,
    UserPaymentNoReason,
    CrimeRobbery,
    BalSnoop,
    UserPaymentWithReason(String),
}

/// A receipt of a transfer.
#[derive(Debug)]
pub(crate) struct DointTransferReceipt {
    /// Who paid the doints.
    pub(crate) sender: DointTransferParty,

    /// Who got the doints.
    pub(crate) recipient: DointTransferParty,

    /// How much the sender sent. (Does not include fees if applicable)
    pub(crate) amount_sent: BigDecimal,

    /// How much the sender spent on fees (if applicable).
    pub(crate) fees_paid: Option<BigDecimal>,

    /// Why this transfer happened.
    pub(crate) transfer_reason: DointTransferReason,
    // What time this transaction occurred at
    // TODO: add me when we track this for a log/ledger
}

/// Error type for Doint transfers.
#[derive(Error, Debug)]
pub(crate) enum DointTransferError {
    #[error(
        "The sender doesn't have enough Doints to cover the transaction, and possibly its fees."
    )]
    SenderInsufficientFunds(DointTransferSenderBroke),

    #[error("The recipient doesn't have room for the incoming funds.")]
    RecipientFull,

    #[error("At least one of the parties involved in the transfer does not exist.")]
    InvalidParty,

    #[error("Attempted to make a transfer from the bank with fees enabled.")]
    TransferFeesOnBank,

    #[error("Cannot transfer funds from a party, to that same party. Cannot transfer 0 doints.")]
    PointlessTransfer,

    #[error("Casting the numbers around failed. Transfer must be <= u32")]
    TransferTooBig,

    #[error("The picked reason for the transfer is incompatible with other arguments.")]
    InvalidTransferReason,

    #[error("Other diesel related errors.")]
    DieselError(#[from] diesel::result::Error),
}

/// Couldn't afford the transfer, here's the breakdown.
#[derive(Debug)]
pub(crate) struct DointTransferSenderBroke {
    /// How much the transfer was worth
    pub(crate) transfer_amount: BigDecimal,

    /// How much in fee's the user would've needed to pay. (if applicable)
    pub(crate) fees_required: Option<BigDecimal>,
}

//
// Now for the function that actually does the transfer
//

impl BankInterface {
    /// Transfer funds between two parties.
    ///
    /// Requires a `DointTransfer`.
    ///
    /// Returns a receipt.
    pub(crate) fn bank_transfer(
        conn: &mut MysqlConnection,
        transfer: DointTransfer,
    ) -> Result<DointTransferReceipt, DointTransferError> {
        run_bank_transfer(conn, transfer)
    }
}

#[allow(clippy::too_many_lines)] // See todo
fn run_bank_transfer(
    conn: &mut MysqlConnection,
    transfer: DointTransfer,
) -> Result<DointTransferReceipt, DointTransferError> {
    // First, do checks on the transfer to make sure its even attemptable.

    // TODO: Somehow extract these checks out into the DointTransfer creation process, and make
    // impossible combinations un-representable instead of doing all of these checks.

    //
    // //
    // Transaction checks
    // //
    //

    // Make sure the two parties are distinct.
    if transfer.sender == transfer.recipient {
        warn!("Attempted to send money between self and self! Pointless!");
        return Err(DointTransferError::PointlessTransfer);
    }

    // Can't transfer nothing
    if transfer.transfer_amount == BigDecimal::zero() {
        warn!("Attempted to move 0 doints between parties! Pointless!");
        return Err(DointTransferError::PointlessTransfer);
    }

    // Fees cannot be enabled if the sender is the bank
    if transfer.sender == DointTransferParty::Bank && transfer.apply_fees {
        return Err(DointTransferError::TransferFeesOnBank);
    }

    // If fees are enabled, calculate them and add them to the transfer
    let fees = if transfer.apply_fees {
        BankInterface::calculate_fees(conn, &transfer.transfer_amount)?
    } else {
        // no fee
        BigDecimal::zero()
    };

    // Cast that the same type the bal is stored as in the DB.
    let full_sender_spend: BigDecimal = &transfer.transfer_amount + &fees;

    // If this is a transfer between 2 users, assert it as such
    if transfer.transfer_reason == DointTransferReason::UserPaymentNoReason
        || matches!(
            transfer.transfer_reason,
            DointTransferReason::UserPaymentWithReason(_)
        )
    {
        // Make sure both parties are users
        match transfer.sender {
            DointTransferParty::DointUser(_) => {}
            _ => {
                // Not a user
                return Err(DointTransferError::InvalidTransferReason);
            }
        }
        match transfer.recipient {
            DointTransferParty::DointUser(_) => {}
            _ => {
                // Not a user
                return Err(DointTransferError::InvalidTransferReason);
            }
        }
        // All good.
    }

    // Taxes can only be collected from users, into the bank.
    if transfer.transfer_reason == DointTransferReason::TaxCollection {
        match transfer.sender {
            DointTransferParty::DointUser(_) => {}
            _ => {
                // Taxes have to come from users.
                return Err(DointTransferError::InvalidTransferReason);
            }
        }

        match transfer.recipient {
            DointTransferParty::Bank => {}
            _ => {
                // Taxes have to go into the bank
                return Err(DointTransferError::InvalidTransferReason);
            }
        }
    }

    // UBI can only come from the bank, to the users.
    if transfer.transfer_reason == DointTransferReason::TaxCollection {
        match transfer.sender {
            DointTransferParty::Bank => {}
            _ => {
                // Taxes have to go into the bank
                return Err(DointTransferError::InvalidTransferReason);
            }
        }

        match transfer.recipient {
            DointTransferParty::DointUser(_) => {}
            _ => {
                // Taxes have to come from users.
                return Err(DointTransferError::InvalidTransferReason);
            }
        }
    }

    //
    // //
    // End of checks
    // //
    //

    // pre-make the response if the sender cannot afford it.
    let sender_cant_afford =
        DointTransferError::SenderInsufficientFunds(DointTransferSenderBroke {
            transfer_amount: transfer.transfer_amount.clone(),
            fees_required: if transfer.apply_fees {
                Some(fees.clone())
            } else {
                None
            },
        });

    // Make sure that both parties exist, and that the transfer can happen.
    match transfer.sender {
        DointTransferParty::Bank => {
            // Bank must exist.
            // Check if bank has funds
            let bal = BankInterface::get_bank_balance(conn)?;

            // Bal must be positive.
            if bal <= BigDecimal::zero() {
                return Err(sender_cant_afford);
            }

            // We now know the bal is positive.

            // Can bank afford it?
            if bal < full_sender_spend {
                return Err(sender_cant_afford);
            }

            // All good.
        }
        DointTransferParty::DointUser(id) => {
            let Some(user) = Users::get_doint_user(id, conn)? else {
                // Couldn't find them
                return Err(DointTransferError::InvalidParty);
            };

            // Do we have enough money
            if user.bal < full_sender_spend {
                return Err(sender_cant_afford);
            }
        }
    }
    match transfer.recipient {
        DointTransferParty::Bank => {
            // Bank must exist.
            // Make sure it has room... Wait how does big-int behave if a number cant fit???
            // TODO: ^
            // let bal = BankInterface::get_bank_balance(conn)?;
            // if bal.checked_add_unsigned(transfer.transfer_amount).is_none() {
            //     // Bank is too full.
            //     return Err(DointTransferError::RecipientFull)
            // }
        }
        DointTransferParty::DointUser(id) => {
            let Some(user) = Users::get_doint_user(id, conn)? else {
                // User does not exist.
                return Err(DointTransferError::InvalidParty);
            };
            // Have room?
            // TODO: Same reason as above.
            // if user.bal.checked_add_unsigned(transfer.transfer_amount).is_none() {
            //     // Ough... im so full...
            //     return Err(DointTransferError::RecipientFull)
            // }
        }
    }

    // Enter a transaction, everything past this point is an operation that would need
    // to be rolled back
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        // Take money from the sender
        match transfer.sender {
            DointTransferParty::Bank => {
                // Take money from bank
                let mut the_bank: BankInfo = bank.first(conn)?;
                the_bank.doints_on_hand -= full_sender_spend;
                the_bank.save_changes::<BankInfo>(conn)?;
            }
            DointTransferParty::DointUser(id) => {
                // Take money from a user
                let mut user = Users::get_doint_user(id, conn)?.expect("Already checked.");
                user.bal -= full_sender_spend;
                user.save_changes::<DointUser>(conn)?;
            }
        }

        // Give that money to the recipient
        match transfer.recipient {
            DointTransferParty::Bank => {
                let mut the_bank: BankInfo = bank.first(conn)?;
                the_bank.doints_on_hand += &transfer.transfer_amount;
                the_bank.save_changes::<BankInfo>(conn)?;
            }
            DointTransferParty::DointUser(id) => {
                let mut user = Users::get_doint_user(id, conn)?.expect("Already checked.");
                user.bal += &transfer.transfer_amount;
                user.save_changes::<DointUser>(conn)?;
            }
        }

        // Put fees in the bank if needed
        if transfer.apply_fees {
            let mut the_bank: BankInfo = bank.first(conn)?;
            the_bank.doints_on_hand += &fees;
            the_bank.save_changes::<BankInfo>(conn)?;
        }

        // Done.
        Ok(())
    })?;

    // Now return a receipt.
    Ok(DointTransferReceipt {
        sender: transfer.sender,
        recipient: transfer.recipient,
        amount_sent: transfer.transfer_amount,
        fees_paid: {
            if transfer.apply_fees {
                Some(fees)
            } else {
                None
            }
        },
        transfer_reason: transfer.transfer_reason,
    })
}
