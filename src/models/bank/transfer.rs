// Move doints from one place to another, wether that be between users or between the bank and elsewhere.

use bigdecimal::{BigDecimal, Zero};
use diesel::{Connection, MysqlConnection};
use thiserror::Error;

use crate::prelude::*;
use diesel::prelude::*;

/// Struct for facilitating doint transfers between two parties.
///
/// Please read the documentation of the struct fields for requirements.
pub struct DointTransfer {
    /// Where the doints are coming from.
    pub sender: DointTransferParty,

    /// Where the doints are going to.
    pub recipient: DointTransferParty,

    /// The amount of doints being sent.
    ///
    /// This must be a positive number. If you wish to take doints from one place, simply swap
    /// the order of sender and recipient.
    pub transfer_amount: BigDecimal,

    /// Do fees apply to this transaction?
    ///
    /// For example, you shouldn't be collecting transfer fees while collecting taxes.
    ///
    /// Transfers out of the bank cannot incur transfer fees, that would be pointless.
    pub apply_fees: bool,

    /// Why this transfer is being made.
    ///
    /// User payments must happen between 2 users, no other combinations are allowed.
    pub transfer_reason: DointTransferReason,
}

impl DointTransfer {
    pub fn new(
        sender: DointTransferParty,
        recipient: DointTransferParty,
        transfer_amount: BigDecimal,
        apply_fees: bool,
        transfer_reason: DointTransferReason,
    ) -> Result<Self, DointTransferConstructionError> {
        // Check for ExtraneousTransfers
        if sender == recipient {
            return Err(DointTransferConstructionError::ExtraneousTransfer(
                "Same party".into(),
            ));
        }

        if transfer_amount == BigDecimal::zero() {
            return Err(DointTransferConstructionError::ExtraneousTransfer(
                "Zero transfer".into(),
            ));
        }

        if sender == DointTransferParty::Bank && apply_fees {
            return Err(DointTransferConstructionError::TransferFeesOnBank);
        }

        // Check for InvalidTransferReasons
        match transfer_reason {
            DointTransferReason::GenericUserPayment
            | DointTransferReason::SpecificUserPayment(_) => {
                if !sender.is_user() || !sender.is_user() {
                    return Err(DointTransferConstructionError::InvalidTransferReason);
                }
            }
            DointTransferReason::TaxCollection => {
                if sender.is_bank() {
                    return Err(DointTransferConstructionError::InvalidTransferReason);
                }
            }
            DointTransferReason::UniversalBasicIncome => {
                if sender.is_user() || recipient.is_bank() {
                    return Err(DointTransferConstructionError::InvalidTransferReason);
                }
            }
            _ => {}
        }

        Ok(Self {
            sender,
            recipient,
            apply_fees,
            transfer_amount,
            transfer_reason,
        })
    }
}

/// Enum for picking where doints are being transferred to.
#[derive(PartialEq, Eq, Debug)]
pub enum DointTransferParty {
    /// The central Doint bank.
    Bank,
    /// A user. Must provide their discord user ID.
    DointUser(u64),
}

impl DointTransferParty {
    pub fn is_user(&self) -> bool {
        matches!(self, DointTransferParty::DointUser(_))
    }

    pub fn is_bank(&self) -> bool {
        matches!(self, DointTransferParty::Bank)
    }
}

/// Why this transfer is occurring (for logging and such)
#[derive(PartialEq, Eq, Debug)]
pub enum DointTransferReason {
    TaxCollection,
    CasinoLoss,
    CasinoWin,
    UniversalBasicIncome,
    GenericUserPayment,
    CrimeRobbery,
    BalSnoop,
    SpecificUserPayment(String),
}

/// A receipt of a transfer.
#[derive(Debug, PartialEq)]
pub struct DointTransferReceipt {
    /// Who paid the doints.
    pub sender: DointTransferParty,

    /// Who got the doints.
    pub recipient: DointTransferParty,

    /// How much the sender sent. (Does not include fees if applicable)
    pub amount_sent: BigDecimal,

    /// How much the sender spent on fees (if applicable).
    pub fees_paid: Option<BigDecimal>,

    /// Why this transfer happened.
    pub transfer_reason: DointTransferReason,
    // What time this transaction occurred at
    // TODO: add me when we track this for a log/ledger
}

/// Error type for Doint transfers.
#[derive(Error, Debug)]
pub enum DointTransferError {
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

    #[error("Cannot transfer funds to the same party.")]
    SameParty,

    #[error("Cannot transfer zero doints")]
    ZeroTransfer,

    #[error("Casting the numbers around failed. Transfer must be <= u32")]
    TransferTooBig,

    #[error("The picked reason for the transfer is incompatible with other arguments.")]
    InvalidTransferReason,

    #[error("Other diesel related errors.")]
    DieselError(#[from] diesel::result::Error),
}

#[derive(Error, Debug)]
pub enum DointTransferConstructionError {
    #[error("At least one of the parties involved in the transfer does not exist.")]
    InvalidParty,

    #[error("Attempted to make a transfer from the bank with fees enabled.")]
    TransferFeesOnBank,

    #[error("Extraneous transfer: {0}")]
    ExtraneousTransfer(String),

    #[error("The picked reason for the transfer is incompatible with other arguments.")]
    InvalidTransferReason,
}

/// Couldn't afford the transfer, here's the breakdown.
#[derive(Debug)]
pub struct DointTransferSenderBroke {
    /// How much the transfer was worth
    pub transfer_amount: BigDecimal,

    /// How much in fee's the user would've needed to pay. (if applicable)
    pub fees_required: Option<BigDecimal>,
}

impl BankInterface {
    /// Transfer funds between two parties.
    ///
    /// Requires a `DointTransfer`.
    ///
    /// Returns a receipt.
    pub fn bank_transfer(
        conn: &mut MysqlConnection,
        transfer: DointTransfer,
    ) -> Result<DointTransferReceipt, DointTransferError> {
        run_bank_transfer(conn, transfer)
    }
}

fn run_bank_transfer(
    conn: &mut MysqlConnection,
    transfer: DointTransfer,
) -> Result<DointTransferReceipt, DointTransferError> {
    // If fees are enabled, calculate them and add them to the transfer
    let fees = if transfer.apply_fees {
        BankInterface::calculate_fees(conn, &transfer.transfer_amount)?
    } else {
        BigDecimal::zero()
    };

    // Cast the bal to the type that is stored in the DB.
    let transfer_with_fees: BigDecimal = &transfer.transfer_amount + &fees;

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
            // Can the bank afford it?
            if bal < transfer_with_fees {
                return Err(sender_cant_afford);
            }
        }
        DointTransferParty::DointUser(id) => {
            let Some(user) = Users::get_doint_user(id, conn)? else {
                // Couldn't find them
                return Err(DointTransferError::InvalidParty);
            };

            // Do we have enough money
            if user.bal < transfer_with_fees {
                return Err(sender_cant_afford);
            }
        }
    }

    // If the recipient is a user, make sure they exist
    if transfer.recipient.is_user()
        && let DointTransferParty::DointUser(id) = transfer.recipient
        && Users::get_doint_user(id, conn)?.is_none()
    {
        // If the party doesn't exist, still try to charge the sender for wasting our time.
        conn.transaction::<(), diesel::result::Error, _>(|conn| {
            if let DointTransferParty::DointUser(sender_id) = transfer.sender {
                let mut sender =
                    Users::get_doint_user(sender_id, conn)?.expect("Sender should be valid");
                sender.bal -= &fees;

                let mut the_bank: BankInfo = bank_table.first(conn)?;
                the_bank.doints_on_hand += &fees;

                the_bank.save_changes::<BankInfo>(conn)?;
                sender.save_changes::<DointUser>(conn)?;
            }

            Ok(())
        })?;
        return Err(DointTransferError::InvalidParty);
    }

    // Enter a transaction, everything past this point is an operation that would need
    // to be rolled back
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        // Take money from the sender
        match transfer.sender {
            DointTransferParty::Bank => {
                // Take money from bank
                let mut the_bank: BankInfo = bank_table.first(conn)?;
                the_bank.doints_on_hand -= transfer_with_fees;
                the_bank.save_changes::<BankInfo>(conn)?;
            }
            DointTransferParty::DointUser(id) => {
                // Take money from a user
                let mut user = Users::get_doint_user(id, conn)?.expect("Already checked.");
                user.bal -= transfer_with_fees;
                user.save_changes::<DointUser>(conn)?;
            }
        }

        // Give that money to the recipient
        match transfer.recipient {
            DointTransferParty::Bank => {
                let mut the_bank: BankInfo = bank_table.first(conn)?;
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
            let mut the_bank: BankInfo = bank_table.first(conn)?;
            the_bank.doints_on_hand += &fees;
            the_bank.save_changes::<BankInfo>(conn)?;
        }

        // Done.
        Ok(())
    })?;

    // Return a reciept of what changed
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
