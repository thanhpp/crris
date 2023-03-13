// @generated automatically by Diesel CLI.

diesel::table! {
    messages (timestamp) {
        timestamp -> Int8,
        username -> Text,
        message -> Text,
    }
}
