use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: i32,
    pub quote_id: Option<i32>,
    pub send_time: NaiveDateTime,
    pub read_time: Option<NaiveDateTime>,
    pub from_user: i32,
    pub to_user: i32,
    pub message_type: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPageModel {
    pub page: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageModel {
    pub quote_id: Option<i32>,
    pub to_user: i32,
    pub message_type: i32,
    pub message: String,
}

#[derive(actix::Message, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub struct StreamMessage {
    pub id: i32,
    pub user_id: i32,
    pub quote_id: Option<i32>,
    pub send_time: NaiveDateTime,
    pub message_type: i32,
    pub message: String,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id: i32,
    pub addr: actix::Recipient<StreamMessage>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i32,
}
