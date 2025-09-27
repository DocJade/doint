// no bank runs please.

use crate::bank::movement::move_doints::{
    DointTransfer, DointTransferError, DointTransferParty, DointTransferReason,
};
use crate::database::tables::fees::FeeInfo;
use crate::tests::setup::get_test_db;
use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};
use diesel::Connection;
use log::info;

use crate::bank::bank_struct::BankInterface;
use crate::database::tables::bank::BankInfo;
use crate::database::tables::users::DointUser;
use crate::schema::bank::dsl::bank;
use crate::schema::fees::dsl::fees;
use crate::schema::users::dsl::users;
use diesel::prelude::*;

#[test]
/// All tests related to the /pay command
#[allow(clippy::too_many_lines)] // its a test mate.
fn test_pay_slash_command() {
    // We cant use the command itself since we dont have a discord context, so yeahhh

    // Open the DB
    get_test_db().test_transaction(|conn| {
        // We are now testing.
        // Make 2 test users to do transfers between
        let mut user1 = DointUser {
            id: 1,
            bal: BigDecimal::from_usize(1000).unwrap(),
        };
        let mut user2 = DointUser {
            id: 2,
            bal: BigDecimal::from_usize(1000).unwrap(),
        };

        user1
            .clone()
            .insert_into(users)
            .execute(conn)
            .expect("Failed to insert user 1");
        user2
            .clone()
            .insert_into(users)
            .execute(conn)
            .expect("Failed to insert user 2");

        // Set up the bank to a known state
        let mut the_bank: BankInfo = bank.first(conn).expect("Failed to get bank");
        the_bank.doints_on_hand = BigDecimal::zero();
        the_bank.total_doints = BigDecimal::from_usize(1_000_000).unwrap();
        the_bank
            .save_changes::<BankInfo>(conn)
            .expect("Couldn't set bank to known values.");

        // Set fees to known, easy to pre-calc (calculate) state.
        let mut the_fees: FeeInfo = fees.first(conn).expect("Failed to get bank");
        the_fees.flat_fee = BigDecimal::one(); // 1 doint
        the_fees.percentage_fee = 10; // 1%
        the_fees
            .save_changes::<FeeInfo>(conn)
            .expect("Couldn't set fees to known values.");

        // Now for the actual test transactions.

        // We can go back and forth with the transactions.

        // Payments of 0 should fail.
        let zero_transfer = DointTransfer {
            sender: DointTransferParty::DointUser(user1.id),
            recipient: DointTransferParty::DointUser(user2.id),
            transfer_amount: BigDecimal::zero(),
            apply_fees: true,
            transfer_reason: DointTransferReason::UserPaymentNoReason,
        };

        match BankInterface::bank_transfer(conn, zero_transfer) {
            Ok(ok) => {
                // This should fail.
                panic!("0 Doint transfer up worked! {ok:#?}")
            }
            Err(err) => match err {
                DointTransferError::PointlessTransfer => {
                    // This is the intended outcome.
                    info!("Zero transfer failed correctly.");
                }
                _ => {
                    // Failed in the wrong way
                    panic!("zero transfer failed, but for the wrong reason! {err:#?}")
                }
            },
        }

        // The maxium possible fee for a bal of 1000 should be 11.
        // Therefore all transfers between 1 and 989 should work.
        // Transfer back and forth so no change happens.

        for amount in 1..=989 {
            let one_way = DointTransfer {
                sender: DointTransferParty::DointUser(user1.id),
                recipient: DointTransferParty::DointUser(user2.id),
                transfer_amount: BigDecimal::from_i32(amount).unwrap(),
                apply_fees: true,
                transfer_reason: DointTransferReason::UserPaymentNoReason,
            };

            let the_other = DointTransfer {
                sender: DointTransferParty::DointUser(user2.id),
                recipient: DointTransferParty::DointUser(user1.id),
                transfer_amount: BigDecimal::from_i32(amount).unwrap(),
                apply_fees: true,
                transfer_reason: DointTransferReason::UserPaymentNoReason,
            };

            let user1_sent = BankInterface::bank_transfer(conn, one_way)
                .expect("In-range transfer failed.")
                .amount_sent;
            let user2_sent = BankInterface::bank_transfer(conn, the_other)
                .expect("In-range transfer failed.")
                .amount_sent;

            assert_eq!(
                user1_sent,
                BigDecimal::from_i32(amount).unwrap(),
                "Transfer amount incorrect"
            );
            assert_eq!(
                user1_sent, user2_sent,
                "Amounts sent were different somehow"
            );

            // Now we also have to immediately reset the bal of the users, since the bank kept the fees
            user1.bal = BigDecimal::from_usize(1000).unwrap();
            user2.bal = BigDecimal::from_usize(1000).unwrap();

            user1
                .save_changes::<DointUser>(conn)
                .expect("Failed to re-fill test user1's money!");
            user2
                .save_changes::<DointUser>(conn)
                .expect("Failed to re-fill test user2's money!");
        }

        // Now anything higher than that should fail
        for amount in 990..2000 {
            let bad = DointTransfer {
                sender: DointTransferParty::DointUser(user1.id),
                recipient: DointTransferParty::DointUser(user2.id),
                transfer_amount: BigDecimal::from_i32(amount).unwrap(),
                apply_fees: true,
                transfer_reason: DointTransferReason::UserPaymentNoReason,
            };

            match BankInterface::bank_transfer(conn, bad) {
                Ok(ok) => {
                    // This shouldn't work
                    panic!("How did we afford that? {ok:#?}")
                }
                Err(err) => match err {
                    DointTransferError::SenderInsufficientFunds(_) => {
                        // This is good.
                    }
                    _ => {
                        // It should fail in the one way, not another (im gonna getcha getcha getcha)
                        panic!("{err:#?}")
                    }
                },
            }
        }

        // What if we pay someone who doesnt exist?
        // This should fail
        let nobody_is_home = DointTransfer {
            sender: DointTransferParty::DointUser(user1.id),
            recipient: DointTransferParty::DointUser(0),
            transfer_amount: BigDecimal::from_usize(50).unwrap(),
            apply_fees: true,
            transfer_reason: DointTransferReason::UserPaymentNoReason,
        };

        let nobody_transfer =
            BankInterface::bank_transfer(conn, nobody_is_home).expect_err("This should fail.");
        match nobody_transfer {
            DointTransferError::InvalidParty => {
                // This is the expected outcome.
            }
            _ => {
                // Wrong kinda error boi
                panic!("Nobody transfer failed for wrong reason! {nobody_transfer:#?}")
            }
        }

        // After all of that, the bank should have collected some fees.
        let the_bank: BankInfo = bank.first(conn).expect("Failed to get bank");
        assert!(
            the_bank.doints_on_hand > BigDecimal::zero(),
            "Fees were not collected!"
        );

        // All tests passed.
        Ok::<(), ()>(())
    });
}
