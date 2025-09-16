// The user table's entries.

use diesel::prelude::*;
use bigdecimal::BigDecimal;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct DointUser {
    pub id: u64,
    pub bal: i32,
}