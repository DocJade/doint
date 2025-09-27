// Fees pease!

use bigdecimal::BigDecimal;
use diesel::prelude::*;

#[derive(Queryable, Selectable, AsChangeset, Identifiable, Clone, Debug)]
#[diesel(table_name = crate::schema::fees)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct FeeInfo {
    /// Used to lock the table to one row.
    ///
    /// Useless, hence private
    id: String,

    /// How many doints every transaction must pay, regardless of transaction size.
    pub flat_fee: BigDecimal,

    /// A percentage based fee that is applied on top of the flat fee.
    ///
    /// Expressed as a percentage.
    ///
    /// Examples:
    /// 1000: 100.0% fee, a action of 100 doints would get a fee of 100 added.
    ///
    /// 100: 10.0%
    ///
    /// 10: 1.0%
    ///
    /// 1: 0.1%
    pub percentage_fee: i16,
}
