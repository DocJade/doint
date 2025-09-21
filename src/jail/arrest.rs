// Time to go to jail!
// We will also add this method to the user type itself.

use chrono::{Local, NaiveDateTime, TimeDelta};
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use crate::schema::jail::dsl::jail;

use crate::{database::tables::{jail::JailedUser, users::DointUser}, jail::{error::JailError, reasons::{JailCause, JailReason}}};

/// The form to fill out when jailing someone.
pub(crate) struct JailForm {
    /// What crime was committed
    pub(crate) law_broke: JailReason,
    
    /// Who/what is sending this user to jail
    pub(crate) arrested_by: JailCause,

    /// How long this user should be in jail for.
    /// 
    /// If set to `None`, duration will be calculated based on the crime.
    pub(crate) jail_for: Option<TimeDelta>,

    /// Is this user eligible for bail?
    pub(crate) can_bail: bool
}

// Impl it on DointUser for ease of use.
impl DointUser {
    /// Puts a user in jail for the specified reason.
    /// 
    /// If user is already in jail, return their current jail status in the error.
    pub(crate) fn jail_user(self, form: &JailForm, conn: &mut MysqlConnection) -> Result<(), JailError> {
        put_user_in_jail(self, form, conn)
    }
}



// actual implementation
fn put_user_in_jail(user: DointUser, form: &JailForm, conn: &mut MysqlConnection) -> Result<(), JailError> {
    // Make sure they aren't already in jail
    if let Some(in_jail) = user.is_jailed(conn)? {
        // User is already in jail.
        return Err(JailError::AlreadyInJail(in_jail))
    }

    // User is not in jail. Put em in!
    
    // If the jailing duration is not set, get the default based on the crime.
    let release_time: NaiveDateTime = if let Some(pre_set) = form.jail_for {
        // Already set.
        Local::now().checked_add_signed(pre_set).expect("Durations shouldn't be too long.").naive_utc()
    } else {
        // go get it
        let do_the_time = form.law_broke.to_time();
        Local::now().checked_add_signed(do_the_time).expect("Durations shouldn't be too long.").naive_utc()
    };


    // Create the jailed user
    let jailed_user: JailedUser = JailedUser {
        id: user.id,
        until: release_time,
        reason: form.law_broke,
        cause: form.arrested_by,
        can_bail: form.can_bail,
    };

    // Put them in the DB
    conn.transaction(|conn|{
        diesel::insert_into(jail).values(jailed_user).execute(conn)
    })?;

    // Jailed!

    Ok(())
}