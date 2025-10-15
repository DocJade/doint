#[cfg(test)]
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
        the_fees.percentage_fee = 100;
        the_fees
            .save_changes::<FeeInfo>(conn)
            .expect("Couldn't set fees to known values.");

        (the_bank, the_fees)
    }

    fn get_bank(conn: &mut MysqlConnection) -> BankInfo {
        bank_table.first(conn).expect("Failed to get bank!")
    }

    #[tokio::test]
    async fn user_to_user() {
        let mut conn = get_isolated_test_db().await;

        let transfer_amount = BigDecimal::from_i32(50).unwrap();

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let user_a = create_test_user(conn);
            let user_b = create_test_user(conn);

            setup_bank_and_fees(conn);

            let transfer = DointTransfer::new(
                DointTransferParty::DointUser(user_a.id),
                DointTransferParty::DointUser(user_b.id),
                transfer_amount.clone(),
                true,
                DointTransferReason::GenericUserPayment,
            )
            .expect("Transfer should be valid");

            let reciept =
                BankInterface::bank_transfer(conn, transfer).expect("Transfer should succeed!");

            let fees_paid = BankInterface::calculate_fees(conn, &transfer_amount).unwrap();

            assert_eq!(
                reciept,
                DointTransferReceipt {
                    amount_sent: transfer_amount.clone(),
                    fees_paid: Some(fees_paid.clone()),
                    recipient: DointTransferParty::DointUser(user_b.id),
                    sender: DointTransferParty::DointUser(user_a.id),
                    transfer_reason: DointTransferReason::GenericUserPayment,
                }
            );

            // Get the data again since it has changed
            let the_bank = get_bank(conn);
            let user_a = Users::get_doint_user(user_a.id, conn)?.expect("User should exist!");
            let user_b = Users::get_doint_user(user_b.id, conn)?.expect("User should exist!");

            assert_eq!(the_bank.doints_on_hand, fees_paid);

            assert_eq!(
                user_a.bal,
                BigDecimal::from_u64(1000).unwrap() - (transfer_amount.clone() + fees_paid)
            );
            assert_eq!(
                user_b.bal,
                BigDecimal::from_u64(1000).unwrap() + transfer_amount.clone()
            );

            Ok(())
        })
    }

    #[tokio::test]
    async fn user_to_none() {
        let mut conn = get_isolated_test_db().await;

        let transfer_amount = BigDecimal::from_i32(50).unwrap();

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let user_a = create_test_user(conn);

            setup_bank_and_fees(conn);

            let transfer = DointTransfer::new(
                DointTransferParty::DointUser(user_a.id),
                DointTransferParty::DointUser(0),
                transfer_amount.clone(),
                true,
                DointTransferReason::GenericUserPayment,
            )
            .expect("Transfer should be valid");

            let error =
                BankInterface::bank_transfer(conn, transfer).expect_err("Transfer should fail!");

            assert!(
                matches!(error, DointTransferError::InvalidParty),
                "Should be an InvalidParty error!"
            );

            // Get the data again since it has changed
            let the_bank = get_bank(conn);
            let user_a = Users::get_doint_user(user_a.id, conn)?.expect("User should exist!");

            assert_eq!(the_bank.doints_on_hand, BigDecimal::zero());
            assert_eq!(user_a.bal, BigDecimal::from_u64(1000).unwrap());

            Ok(())
        })
    }

    #[tokio::test]
    async fn bank_to_user() {
        let mut conn = get_isolated_test_db().await;

        let transfer_amount = BigDecimal::from_i32(50).unwrap();

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let user_a = create_test_user(conn);

            let (mut the_bank, _) = setup_bank_and_fees(conn);
            the_bank.doints_on_hand = BigDecimal::from_u64(50).unwrap();
            the_bank
                .save_changes::<BankInfo>(conn)
                .expect("Expected balance change to succeed");

            let transfer = DointTransfer::new(
                DointTransferParty::Bank,
                DointTransferParty::DointUser(user_a.id),
                transfer_amount.clone(),
                false,
                DointTransferReason::UniversalBasicIncome,
            )
            .expect("Transfer should be valid");

            let reciept =
                BankInterface::bank_transfer(conn, transfer).expect("Transfer should succeed!");

            assert_eq!(
                reciept,
                DointTransferReceipt {
                    amount_sent: transfer_amount.clone(),
                    fees_paid: None,
                    recipient: DointTransferParty::DointUser(user_a.id),
                    sender: DointTransferParty::Bank,
                    transfer_reason: DointTransferReason::UniversalBasicIncome,
                }
            );

            // Get the data again since it has changed
            let the_bank = get_bank(conn);
            let user_a = Users::get_doint_user(user_a.id, conn)?.expect("User should exist!");

            assert_eq!(the_bank.doints_on_hand, BigDecimal::from_u64(0).unwrap());

            assert_eq!(
                user_a.bal,
                BigDecimal::from_u64(1000).unwrap() + transfer_amount.clone()
            );

            Ok(())
        })
    }

    #[tokio::test]
    async fn bank_to_none() {
        let mut conn = get_isolated_test_db().await;

        let transfer_amount = BigDecimal::from_i32(50).unwrap();

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let (mut the_bank, _) = setup_bank_and_fees(conn);
            the_bank.doints_on_hand = BigDecimal::from_u64(50).unwrap();
            the_bank
                .save_changes::<BankInfo>(conn)
                .expect("Expected balance change to succeed");

            let transfer = DointTransfer::new(
                DointTransferParty::Bank,
                DointTransferParty::DointUser(0),
                transfer_amount.clone(),
                false,
                DointTransferReason::UniversalBasicIncome,
            )
            .expect("Transfer should be valid");

            let reciept =
                BankInterface::bank_transfer(conn, transfer).expect_err("Transfer should fail!");

            assert!(matches!(reciept, DointTransferError::InvalidParty));

            // Get the data again since it has changed
            let the_bank = get_bank(conn);

            assert_eq!(the_bank.doints_on_hand, BigDecimal::from_u64(50).unwrap());

            Ok(())
        })
    }

    #[tokio::test]
    /// Tests for a regression where each fee is always 1 dent
    async fn fee_calculation() {
        let mut conn = get_isolated_test_db().await;

        let transfer_amount = BigDecimal::from_i32(10).unwrap();

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let fees_paid = BankInterface::calculate_fees(conn, &transfer_amount).unwrap();

            assert_eq!(fees_paid, BigDecimal::from(2));

            Ok(())
        })
    }
}
