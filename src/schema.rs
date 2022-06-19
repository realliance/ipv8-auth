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
    }
}

joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(sessions, users,);
