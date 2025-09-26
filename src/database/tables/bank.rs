// laughing all the way there. haw haw haw.

use bigdecimal::BigDecimal;
use diesel::prelude::*;

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone, Debug)]
#[diesel(table_name = crate::schema::bank)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct BankInfo {
    /// Used to lock the table to one row.
    /// 
    /// Useless, hence private
    id: String,

    /// How many doints the bank currently has, and can give out.
    /// 
    /// IE these doints are not reserved for any use, and are liquid.
    pub doints_on_hand: BigDecimal,

    /// How many doints are in circulation.
    pub total_doints: BigDecimal,

    /// The current tax rate.
    /// 
    /// Expressed as a percentage.
    /// 
    /// Examples:
    /// 1000: 100.0% tax rate, would take everything from user.
    /// 
    /// 100: 10.0% tax rate.
    /// 
    /// 10: 1.0% tax rate.
    /// 
    /// 1: 0.1% tax rate.
    pub tax_rate: i16,

    /// The percentage of the bank's money that will be spent on UBI every day
    /// 
    /// Expressed in same way as tax rate.
    pub ubi_rate: i16,
}