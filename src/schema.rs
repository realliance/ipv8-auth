table! {
    games (user_id) {
        user_id -> Uuid,
        token -> Uuid,
        instruction -> Int4,
    }
}

table! {
    sessions (token) {
        token -> Uuid,
        user_id -> Uuid,
        last_used -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        username -> Varchar,
        password_digest -> Varchar,
        license_game_stage -> Int4,
    }
}

joinable!(games -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(games, sessions, users,);
