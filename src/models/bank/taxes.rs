use crate::prelude::*;
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{Connection, MysqlConnection};
use log::info;

impl BankInterface {
    /// Immediately collect taxes from all users.
    ///
    /// Taxes are based on a *percentage* of all of your doints at the moment taxes are taken.
    ///
    /// Returns the taxes collected.
    /// Returns a [`DieselError`][diesel::result::Error] if tax collection fails.
    pub fn collect_taxes(conn: &mut MysqlConnection) -> Result<BigDecimal, Error> {
        go_collect_taxes(conn)
    }
}

fn go_collect_taxes(conn: &mut MysqlConnection) -> Result<BigDecimal, Error> {
    info!("Collecting taxes...");

    // If any of this fails, the entire transaction will be rolled back, and taxes will not be collected.
    conn.transaction::<BigDecimal, diesel::result::Error, _>(|conn| {
        // Get the current state of the bank
        let the_bank: BankInfo = bank_table.first(conn)?;

        // If the tax_rate is zero, there's no need to tax people.
        // We check if it's less than 1, since 1 is representative of 0.1%
        if the_bank.tax_rate < 1 {
            info!("Tax rate is zero. Skipping!");
            return Ok(BigDecimal::zero());
        }

        let tax_rate_percentage = conversions::tax_rate_to_percentage(the_bank.tax_rate);

        let tax_rate_multiplier: BigDecimal = BigDecimal::from_f64(tax_rate_percentage)
            .expect("Should be able to represent BigDecimal from f64");

        // Get all users with a positive, non-zero balance
        let mut to_update: Vec<DointUser> = users_table
            .filter(users_bal_table.gt(BigDecimal::zero()))
            .load::<DointUser>(conn)?;

        // Now loop over every user, figuring out how much to take from each of them
        // We also keep track of how much money we have gathered
        let mut collected_taxes: BigDecimal = BigDecimal::zero();
        for user in &mut to_update {
            let adjustment_amount = &user.bal * &tax_rate_multiplier;

            // Round upwards to the nearest dent
            let rounded_adjustment = adjustment_amount.round(2);

            // Cant take more money than they have
            // You must pay in at least 1 doint
            let tax_charge_amount = std::cmp::max(
                std::cmp::min(&user.bal, &rounded_adjustment).clone(),
                BigDecimal::from_u8(1).expect("Should be representable"),
            );

            user.bal -= &tax_charge_amount;

            // This must be a positive number.
            collected_taxes += tax_charge_amount;
        }

        // Apply all of the changes.
        for user in &to_update {
            user.save_changes::<DointUser>(conn)?;
        }

        // Update the bank's balance with the collected taxes.
        let mut update_bank: BankInfo = bank_table.first(conn)?;
        update_bank.doints_on_hand += &collected_taxes;
        update_bank.save_changes::<BankInfo>(conn)?;

        info!("Tax collection finished!");
        info!(
            "Collected [{}] doints via taxes.",
            crate::formatting::format_struct::FormattingHelper::display_doint(&collected_taxes)
        );
        Ok(collected_taxes)
    })
}
