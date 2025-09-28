// The user preferences entry.

// Using JSON in the DB is complicated man

use std::io::Write;

use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable, QueryableByName};
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::mysql::{Mysql, MysqlValue};
use diesel::Selectable;

use crate::database::tables::users::DointUser;
use crate::user::user_settings::DointUserSettings;




// New type to wrap the user settings in
#[derive(Debug, PartialEq, AsExpression, Associations, Identifiable, QueryableByName, Insertable, Clone, AsChangeset)]
#[diesel(belongs_to(DointUser, foreign_key = id))]
#[derive(serde::Serialize)]
#[diesel(sql_type = Text)]
#[diesel(table_name = crate::schema::user_preferences)]
#[primary_key(id)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[allow(private_interfaces)] // This is due to Diesel jank.
pub struct JsonWrapperUserSettings{
    pub id: u64,
    pub settings: DointUserSettings,
}

// Impls to make that work

impl FromSql<Text, Mysql> for DointUserSettings {
    fn from_sql(bytes: MysqlValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        let user_settings: Result<DointUserSettings, serde_json::Error> = serde_json::from_str(&t);
        Ok(user_settings?)
    }
}

impl ToSql<Text, Mysql> for DointUserSettings {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
        out.write_all(serde_json::to_string(&self)?.as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}