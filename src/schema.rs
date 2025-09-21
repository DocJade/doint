// @generated automatically by Diesel CLI.

diesel::table! {
    bank (id) {
        #[max_length = 1]
        id -> Char,
        doints_on_hand -> Integer,
        total_doints -> Integer,
        tax_rate -> Smallint,
        ubi_rate -> Smallint,
    }
}

diesel::table! {
    fees (id) {
        #[max_length = 1]
        id -> Char,
        flat_fee -> Integer,
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
    users (id) {
        id -> Unsigned<Bigint>,
        bal -> Integer,
    }
}

diesel::joinable!(jail -> users (id));

diesel::allow_tables_to_appear_in_same_query!(bank, fees, jail, users,);
