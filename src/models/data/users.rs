// The user table's entries.

use bigdecimal::BigDecimal;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct DointUser {
    pub id: u64,
    pub bal: BigDecimal,
}

impl From<DointUser> for u64 {
    fn from(val: DointUser) -> Self {
        val.id
    }
}
