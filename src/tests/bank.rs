mod bank_tests {
    use crate::{prelude::*, tests::setup::get_isolated_test_db};
    use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};
    use diesel::prelude::*;
    use rand::Rng;

    use diesel::MysqlConnection;

    /// Creates a test user with random ID and 1000 doints
    fn create_test_user(conn: &mut MysqlConnection) -> DointUser {
        let mut rng = rand::rng();

        let user = DointUser {
            id: rng.random::<u64>(),
            bal: BigDecimal::from_usize(1000).unwrap(),
        };

        diesel::insert_into(users_table)
            .values(&user)
            .execute(conn)
            .expect("Failed to insert user");

        user
    }

    /// Resets bank and fees to known state
    fn setup_bank_and_fees(conn: &mut MysqlConnection) -> (BankInfo, FeeInfo) {
        let mut the_bank: BankInfo = bank_table.first(conn).expect("Failed to get bank!");
        the_bank.doints_on_hand = BigDecimal::zero();
        the_bank.total_doints = BigDecimal::from_usize(1_000_000).unwrap();
        the_bank
            .save_changes::<BankInfo>(conn)
            .expect("Couldn't set bank to known values.");

        let mut the_fees: FeeInfo = fees_table.first(conn).expect("Failed to get fees!");
        the_fees.flat_fee = BigDecimal::one();
        the_fees.percentage_fee = 10;
        the_fees
            .save_changes::<FeeInfo>(conn)
            .expect("Couldn't set fees to known values.");

        (the_bank, the_fees)
    }

    #[tokio::test]
    async fn transfer_user_to_user() {
        let mut conn = get_isolated_test_db().await;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let user1 = create_test_user(conn);
            let user2 = create_test_user(conn);
            setup_bank_and_fees(conn);

            for amount in 1..=50 {
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

                assert_eq!(user1_sent, BigDecimal::from_i32(amount).unwrap());
                assert_eq!(user1_sent, user2_sent);

                user1.save_changes::<DointUser>(conn).unwrap();
                user2.save_changes::<DointUser>(conn).unwrap();
            }

            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn transfer_user_to_none() {
        let mut conn = get_isolated_test_db().await;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let user1 = create_test_user(conn);
            let (the_bank, _) = setup_bank_and_fees(conn);

            let none_transfer = DointTransfer {
                sender: DointTransferParty::DointUser(user1.id),
                recipient: DointTransferParty::DointUser(0),
                transfer_amount: BigDecimal::from_usize(50).unwrap(),
                apply_fees: true,
                transfer_reason: DointTransferReason::UserPaymentNoReason,
            };

            let err =
                BankInterface::bank_transfer(conn, none_transfer).expect_err("This should fail.");
            match err {
                DointTransferError::InvalidParty => {}
                _ => panic!("None transfer failed for wrong reason! {err:?}"),
            }

            assert!(
                the_bank.doints_on_hand > BigDecimal::zero(),
                "Fees were not collected!"
            );

            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn sender_insufficient_funds() {
        let mut conn = get_isolated_test_db().await;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let user1 = create_test_user(conn);
            let user2 = create_test_user(conn);
            setup_bank_and_fees(conn);

            for amount in 990..1000 {
                let bad = DointTransfer {
                    sender: DointTransferParty::DointUser(user1.id),
                    recipient: DointTransferParty::DointUser(user2.id),
                    transfer_amount: BigDecimal::from_i32(amount).unwrap(),
                    apply_fees: true,
                    transfer_reason: DointTransferReason::UserPaymentNoReason,
                };

                match BankInterface::bank_transfer(conn, bad) {
                    Ok(ok) => panic!("How did we afford that? {ok:?}"),
                    Err(DointTransferError::SenderInsufficientFunds(_)) => {}
                    Err(err) => panic!("Failed for wrong reason: {err:?}"),
                }
            }

            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn zero_transfer() {
        let mut conn = get_isolated_test_db().await;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let user1 = create_test_user(conn);
            let user2 = create_test_user(conn);
            setup_bank_and_fees(conn);

            let zero_transfer = DointTransfer {
                sender: DointTransferParty::DointUser(user1.id),
                recipient: DointTransferParty::DointUser(user2.id),
                transfer_amount: BigDecimal::zero(),
                apply_fees: true,
                transfer_reason: DointTransferReason::UserPaymentNoReason,
            };

            match BankInterface::bank_transfer(conn, zero_transfer) {
                Ok(ok) => panic!("Zero transfer should fail! {ok:?}"),
                Err(DointTransferError::PointlessTransfer) => {}
                Err(err) => panic!("Failed for wrong reason: {err:?}"),
            }

            Ok(())
        })
        .unwrap();
    }
}
