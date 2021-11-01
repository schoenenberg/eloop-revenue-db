table! {
    configs (key) {
        key -> Text,
        value -> Text,
    }
}

table! {
    net_profits (day) {
        day -> Date,
        value -> Float,
    }
}

table! {
    revenues (day) {
        day -> Date,
        value -> Float,
    }
}

allow_tables_to_appear_in_same_query!(
    configs,
    net_profits,
    revenues,
);
