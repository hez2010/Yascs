use chrono::NaiveDateTime;
use diesel::table;

table! {
    users {
        id -> Integer,
        username -> Text,
        email -> Text,
        phone -> Text,
        avatar -> Text,
        location -> Text,
        age -> Integer,
        gender -> Integer,
        password_hash -> Text,
    }
}

table! {
    messages {
        id -> Integer,
        from_user -> Integer,
        to_user -> Integer,
        quote_id -> Nullable<Integer>,
        message -> Text,
        message_type -> Integer,
        send_time -> Timestamp,
        read_time -> Nullable<Timestamp>,
    }
}

table! {
    friends {
        id -> Integer,
        user_id -> Integer,
        friend_user_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(users, friends, messages);

#[derive(Queryable, Debug, Identifiable, Clone)]
#[table_name = "users"]
#[primary_key(id)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub avatar: String,
    pub location: String,
    pub age: i32,
    pub gender: i32,
    pub password_hash: String,
}

#[derive(Queryable, Debug, Identifiable, Clone)]
#[table_name = "messages"]
#[primary_key(id)]
pub struct Message {
    pub id: i32,
    pub from_user: i32,
    pub to_user: i32,
    pub quote_id: Option<i32>,
    pub message: String,
    pub message_type: i32,
    pub send_time: NaiveDateTime,
    pub read_time: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Identifiable, Clone)]
#[table_name = "friends"]
#[primary_key(id)]
pub struct Friend {
    pub id: i32,
    pub user_id: i32,
    pub friend_user_id: i32,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
}
