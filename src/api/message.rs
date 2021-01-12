use std::collections::HashMap;

use actix::{
    fut, Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, Recipient, Running, StreamHandler, WrapFuture,
};
use actix_identity::Identity;
use actix_web::{error::BlockingError, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use message::{Disconnect, SendMessageModel, StreamMessage};

use crate::{
    model::{
        message::{self, Connect, HistoryPageModel},
        ResultModel,
    },
    schema, DbPool,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/list", web::get().to(list));
    cfg.route("/history/{user_id}", web::get().to(history));
    cfg.route("/send", web::post().to(send));
    cfg.route("/stream", web::get().to(stream));
    cfg.route("/read/{msg_id}", web::post().to(set_read));
}

pub async fn list(identity: Identity, pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.get().expect("Failed to get connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                schema::messages::dsl::messages
                    .filter(
                        schema::messages::dsl::from_user
                            .eq(&self_user_id)
                            .or(schema::messages::dsl::to_user.eq(&self_user_id)),
                    )
                    .select((
                        schema::messages::dsl::id,
                        schema::messages::dsl::quote_id,
                        schema::messages::dsl::read_time,
                        schema::messages::dsl::message_type,
                        schema::messages::dsl::message,
                        schema::messages::dsl::send_time,
                        schema::messages::dsl::from_user,
                        schema::messages::dsl::to_user,
                    ))
                    .order(schema::messages::dsl::send_time.desc())
                    .load::<(
                        i32,
                        Option<i32>,
                        Option<NaiveDateTime>,
                        i32,
                        String,
                        NaiveDateTime,
                        i32,
                        i32,
                    )>(&conn)
            })
            .await
            {
                Ok(result) => {
                    let mut message_map = HashMap::<i32, message::Message>::new();
                    for item in &result {
                        let f = item.clone();
                        let display_user_id = if f.6 == self_user_id { f.7 } else { f.6 };
                        if !message_map.contains_key(&display_user_id) {
                            message_map.insert(
                                display_user_id,
                                message::Message {
                                    id: f.0,
                                    quote_id: f.1,
                                    read_time: f.2,
                                    message_type: f.3,
                                    message: f.4,
                                    send_time: f.5,
                                    from_user: f.6,
                                    to_user: f.7,
                                },
                            );
                        };
                    }

                    let mut message_result = Vec::<message::Message>::new();
                    for item in message_map {
                        message_result.push(item.1);
                    }

                    ResultModel {
                        success: true,
                        code: 200,
                        data: Some(message_result),
                        message: None,
                    }
                }
                Err(BlockingError::Error(e)) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some(e.to_string()),
                },
                Err(BlockingError::Canceled) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some("Operation has been cancelled.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            code: 401,
            data: None,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn history(
    web::Path(user_id): web::Path<i32>,
    web::Query(query): web::Query<HistoryPageModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                schema::messages::dsl::messages
                    .filter(
                        schema::messages::dsl::from_user
                            .eq(&self_user_id)
                            .and(schema::messages::dsl::to_user.eq(&user_id))
                            .or(schema::messages::dsl::from_user
                                .eq(&user_id)
                                .and(schema::messages::dsl::to_user.eq(&self_user_id))),
                    )
                    .order(schema::messages::dsl::send_time.desc())
                    .offset(match query.page {
                        None => 0,
                        Some(page) => ((page - 1) * 10).into(),
                    })
                    .limit(10)
                    .load::<schema::Message>(&conn)
            })
            .await
            {
                Ok(result) => ResultModel {
                    success: true,
                    code: 200,
                    data: Some(
                        result
                            .iter()
                            .map(|item| {
                                let f = item.clone();
                                message::Message {
                                    id: f.id,
                                    quote_id: f.quote_id,
                                    read_time: f.read_time,
                                    message_type: f.message_type,
                                    message: f.message,
                                    send_time: f.send_time,
                                    from_user: f.from_user,
                                    to_user: f.to_user,
                                }
                            })
                            .collect::<Vec<_>>(),
                    ),
                    message: None,
                },
                Err(BlockingError::Error(e)) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some(e.to_string()),
                },
                Err(BlockingError::Canceled) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some("Operation has been cancelled.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            code: 401,
            data: None,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn send(
    web::Json(model): web::Json<SendMessageModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
    stream: web::Data<Addr<MessageStreamServer>>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            if self_user_id == model.to_user {
                return ResultModel::<String> {
                    success: false,
                    code: 400,
                    data: None,
                    message: Some("Cannot send message to yourself.".to_string()),
                };
            }
            match web::block(move || {
                use schema::messages::dsl::*;
                let result = diesel::insert_into(messages)
                    .values((
                        quote_id.eq(&model.quote_id),
                        from_user.eq(&self_user_id),
                        to_user.eq(&model.to_user),
                        message_type.eq(&model.message_type),
                        message.eq(&model.message),
                        send_time.eq(&Utc::now().naive_utc()),
                    ))
                    .get_result::<schema::Message>(&conn);
                if let Ok(ref sent_msg) = result {
                    stream.do_send(TargetStreamMessage {
                        message: StreamMessage {
                            id: sent_msg.id,
                            user_id: sent_msg.from_user,
                            quote_id: sent_msg.quote_id,
                            send_time: sent_msg.send_time,
                            message_type: sent_msg.message_type,
                            message: sent_msg.message.clone(),
                        },
                        user_id: model.to_user,
                    });
                }
                result
            })
            .await
            {
                Ok(_) => ResultModel {
                    success: true,
                    code: 200,
                    data: None,
                    message: None,
                },
                Err(BlockingError::Error(e)) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some(e.to_string()),
                },
                Err(BlockingError::Canceled) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some("Operation has been cancelled.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            code: 401,
            data: None,
            message: Some("Not logged in.".to_string()),
        },
    }
}

#[derive(Clone)]
pub struct MessageStreamServer {
    sessions: HashMap<i32, Recipient<StreamMessage>>,
}

impl MessageStreamServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::<i32, Recipient<StreamMessage>>::new(),
        }
    }
}

impl Actor for MessageStreamServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for MessageStreamServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

impl Handler<Disconnect> for MessageStreamServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct TargetStreamMessage {
    pub user_id: i32,
    pub message: StreamMessage,
}

impl Handler<TargetStreamMessage> for MessageStreamServer {
    type Result = ();

    fn handle(&mut self, msg: TargetStreamMessage, _ctx: &mut Self::Context) -> Self::Result {
        match self.sessions.get(&msg.user_id) {
            Some(session) => session.do_send(msg.message).unwrap(),
            None => (),
        };
    }
}

struct MessageStreamSession {
    pub user_id: i32,
    pub addr: Addr<MessageStreamServer>,
}

impl Actor for MessageStreamSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr
            .send(Connect {
                user_id: self.user_id,
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, _act, ctx| {
                match res {
                    Ok(_) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {
            user_id: self.user_id,
        });
        Running::Stop
    }
}

impl Handler<StreamMessage> for MessageStreamSession {
    type Result = ();

    fn handle(&mut self, msg: StreamMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageStreamSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            _ => (),
        }
    }
}

pub async fn stream(
    req: HttpRequest,
    identity: Identity,
    payload: web::Payload,
    stream: web::Data<Addr<MessageStreamServer>>,
) -> Result<HttpResponse, Error> {
    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            let resp = ws::start(
                MessageStreamSession {
                    user_id: self_user_id,
                    addr: stream.get_ref().clone(),
                },
                &req,
                payload,
            );
            resp
        }
        None => Ok(HttpResponse::Unauthorized()
            .content_type("application/json")
            .body(
                serde_json::to_string(&ResultModel::<String> {
                    success: false,
                    code: 401,
                    data: None,
                    message: Some("Not logged in.".to_string()),
                })
                .unwrap(),
            )
            .into()),
    }
}

pub async fn set_read(
    web::Path(msg_id): web::Path<i32>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                diesel::update(
                    schema::messages::dsl::messages.filter(
                        schema::messages::dsl::id
                            .eq(&msg_id)
                            .and(schema::messages::dsl::to_user.eq(&self_user_id)),
                    ),
                )
                .set(schema::messages::dsl::read_time.eq(Utc::now().naive_utc()))
                .execute(&conn)
            })
            .await
            {
                Ok(_) => ResultModel::<String> {
                    success: true,
                    code: 200,
                    data: None,
                    message: None,
                },
                Err(BlockingError::Error(e)) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some(e.to_string()),
                },
                Err(BlockingError::Canceled) => ResultModel {
                    success: false,
                    code: 500,
                    data: None,
                    message: Some("Operation has been cancelled.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            code: 401,
            data: None,
            message: Some("Not logged in.".to_string()),
        },
    }
}
