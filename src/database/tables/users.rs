// The user table's entries.

use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Identifiable, AsChangeset, Clone, Copy)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct DointUser {
    pub id: u64,
    pub bal: i32,
}