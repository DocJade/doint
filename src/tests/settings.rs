// Im scared of this json db stuff.

use crate::notification::notification_settings::NotificationLocation;
use crate::{database::tables::user_preferences::JsonWrapperUserSettings, user::user_settings::DointUserSettings};
use crate::tests::setup::get_test_db;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{BelongingToDsl, Connection, Insertable, OptionalExtension, RunQueryDsl, SaveChangesDsl, SelectableHelper};

use crate::database::tables::users::DointUser;
use crate::schema::users::dsl::users;
// use diesel::prelude::*;

use diesel::query_dsl::methods::SelectDsl;

use crate::schema::user_preferences::dsl::user_preferences;

#[test]
/// Make sure we can add and remove user preferences
fn test_give_user_preferences() {
    // Open the DB
    get_test_db().test_transaction::<_, (), _>(|conn| {
        // Make a user
        let user1 = DointUser {
            id: 1,
            bal: BigDecimal::from_usize(1000).unwrap(),
        };

        user1
            .clone()
            .insert_into(users)
            .execute(conn)
            .expect("Failed to insert user 1");

        // Now get that user's preferences, they shouldn't exist.

        let not_yet_set: Option<JsonWrapperUserSettings> = JsonWrapperUserSettings::belonging_to(&user1)
        .select(JsonWrapperUserSettings::as_select())
        .first(conn)
        .optional().unwrap();

        // We expect this to be blank
        assert!(not_yet_set.is_none());

        // Go give em some settings

        let default_settings = JsonWrapperUserSettings {
            id: user1.id,
            settings: DointUserSettings::default(),
        };

        // Give it to them
        diesel::insert_into(user_preferences).values(default_settings.clone()).execute(conn).unwrap();
        
        // Now read it back, and it should match
        let mut now_yet_set: JsonWrapperUserSettings = JsonWrapperUserSettings::belonging_to(&user1)
        .select(JsonWrapperUserSettings::as_select())
        .first(conn)
        .optional().unwrap().unwrap();

        // It should match the defaults still
        assert_eq!(now_yet_set.settings, DointUserSettings::default());

        // now we will change a setting
        now_yet_set.settings.notification_settings.crime_notifications.robbed = NotificationLocation::DirectMessage;

        // put it back
        now_yet_set.save_changes::<JsonWrapperUserSettings>(conn).unwrap();

        // Load it back in the value to make sure it changed
        
        let i_am_looking: JsonWrapperUserSettings = JsonWrapperUserSettings::belonging_to(&user1)
        .select(JsonWrapperUserSettings::as_select())
        .first(conn)
        .optional().unwrap().unwrap();

        // Changed!
        assert_ne!(i_am_looking.settings, default_settings.settings);

        Ok(())
    });
}