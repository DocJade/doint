// @generated automatically by Diesel CLI.

diesel::table! {
    bank (id) {
        #[max_length = 1]
        id -> Char,
        doints_on_hand -> Decimal,
        total_doints -> Decimal,
        tax_rate -> Smallint,
        ubi_rate -> Smallint,
    }
}

diesel::table! {
    fees (id) {
        #[max_length = 1]
        id -> Char,
        flat_fee -> Decimal,
        percentage_fee -> Smallint,
    }
}

diesel::table! {
    jail (id) {
        id -> Unsigned<Bigint>,
        until -> Timestamp,
        reason -> Tinytext,
        cause -> Tinytext,
        can_bail -> Bool,
    }
}

diesel::table! {
    user_preferences (id) {
        id -> Unsigned<Bigint>,
        settings -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        bal -> Decimal,
    }
}

diesel::joinable!(jail -> users (id));
diesel::joinable!(user_preferences -> users (id));

diesel::allow_tables_to_appear_in_same_query!(bank, fees, jail, user_preferences, users,);
