use crate::{
    model::{
        user::{self, PasswordUpdateModel, UserInfoUpdateModel},
        ResultModel, SearchModel,
    },
    schema::{self, NewUser},
    DbPool,
};
use actix_identity::Identity;
use actix_web::{error::BlockingError, web, Responder};
use diesel::prelude::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/profiles", web::get().to(profiles));
    cfg.route("/profiles/{user_id}", web::get().to(profiles_with_id));
    cfg.route("/profiles", web::post().to(update_profiles));
    cfg.route("/password", web::post().to(update_password));
    cfg.route("/search", web::get().to(search));
    cfg.route("/login", web::post().to(login));
    cfg.route("/logout", web::post().to(logout));
    cfg.route("/register", web::post().to(register));
    cfg.route("/friends", web::get().to(friends));
    cfg.route("/friends/{user_id}", web::post().to(add_friend));
    cfg.route("/friends/{user_id}", web::delete().to(delete_friend));
}

pub async fn search(
    web::Query(query): web::Query<SearchModel>,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");
    match web::block(move || {
        use schema::users::dsl::*;
        users
            .filter(
                username
                    .like(&query.patterns)
                    .or(email.like(&query.patterns).or(phone.like(&query.patterns))),
            )
            .offset(match query.page {
                None => 0,
                Some(page) => ((page - 1) * 10).into(),
            })
            .limit(10)
            .load::<schema::User>(&conn)
    })
    .await
    {
        Ok(result) => ResultModel {
            success: true,
            data: Some(
                result
                    .iter()
                    .map(|u| user::UserInfo {
                        id: u.id,
                        age: u.age,
                        gender: u.gender,
                        email: u.email.clone(),
                        phone: u.phone.clone(),
                        username: u.username.clone(),
                        location: u.location.clone(),
                        avatar: u.avatar.clone(),
                    })
                    .collect::<Vec<_>>(),
            ),
            code: 200,
            message: None,
        },
        Err(BlockingError::Error(e)) => ResultModel {
            success: false,
            data: None,
            code: 500,
            message: Some(e.to_string()),
        },
        Err(BlockingError::Canceled) => ResultModel {
            success: false,
            data: None,
            code: 500,
            message: Some("Operation has been cancelled.".to_string()),
        },
    }
}

pub async fn login(
    model: web::Json<user::LoginModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");
    let model_username = model.username.clone();
    let result = web::block(move || {
        use schema::users::dsl::*;
        schema::users::dsl::users
            .filter(username.eq(&model_username))
            .select((id, password_hash))
            .first::<(i32, String)>(&conn)
    })
    .await;
    match result {
        Ok(entry) => match bcrypt::verify(&model.password, &entry.1) {
            Ok(true) => {
                identity.remember(entry.0.to_string());
                ResultModel::<String> {
                    success: true,
                    data: None,
                    code: 200,
                    message: None,
                }
            }
            _ => ResultModel {
                success: false,
                data: None,
                code: 401,
                message: Some("Incorrect username or password.".to_string()),
            },
        },
        Err(BlockingError::Error(_)) => ResultModel {
            success: false,
            data: None,
            code: 401,
            message: Some("Incorrect username or password.".to_string()),
        },
        Err(BlockingError::Canceled) => ResultModel {
            success: false,
            data: None,
            code: 500,
            message: Some("Operation has been cancelled.".to_string()),
        },
    }
}

pub async fn logout(identity: Identity) -> impl Responder {
    identity.forget();
    ResultModel::<String> {
        success: true,
        data: None,
        code: 200,
        message: None,
    }
}

pub async fn register(
    model: web::Json<user::RegisterModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if model.confirm_password != model.password {
        return ResultModel::<String> {
            success: false,
            data: None,
            code: 400,
            message: Some("Mismatch between password and confirm-password.".to_string()),
        };
    }
    let conn = pool.get().expect("Failed to get db connection from pool.");
    let result = web::block(move || {
        use schema::users::dsl::*;
        let new_user = NewUser {
            username: &model.username,
            email: &model.email,
            password_hash: &bcrypt::hash(&model.password, bcrypt::DEFAULT_COST)
                .unwrap()
                .to_string(),
        };
        diesel::insert_into(users)
            .values(&new_user)
            .get_result::<schema::User>(&conn)
    })
    .await;
    match result {
        Ok(user) => {
            identity.remember(user.id.to_string());
            ResultModel::<String> {
                success: true,
                data: None,
                code: 200,
                message: None,
            }
        }
        Err(BlockingError::Error(err)) => ResultModel::<String> {
            success: false,
            data: None,
            code: 500,
            message: Some(err.to_string()),
        },
        Err(BlockingError::Canceled) => ResultModel::<String> {
            success: false,
            data: None,
            code: 500,
            message: Some("Operation has been cancelled.".to_string()),
        },
    }
}

pub async fn profiles(identity: Identity, pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            match web::block(move || {
                use schema::users::dsl::*;
                schema::users::dsl::users
                    .filter(id.eq(&user_id_str.parse::<i32>().unwrap()))
                    .first::<schema::User>(&conn)
            })
            .await
            {
                Ok(user) => ResultModel {
                    success: true,
                    data: Some(user::UserInfo {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        phone: user.phone,
                        avatar: user.avatar,
                        location: user.location,
                        age: user.age,
                        gender: user.gender,
                    }),
                    code: 200,
                    message: None,
                },
                Err(_) => ResultModel {
                    success: false,
                    data: None,
                    code: 401,
                    message: Some("Not logged in.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn profiles_with_id(
    web::Path(user_id): web::Path<i32>,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");
    match web::block(move || {
        use schema::users::dsl::*;
        schema::users::dsl::users
            .filter(id.eq(&user_id))
            .first::<schema::User>(&conn)
    })
    .await
    {
        Ok(user) => ResultModel {
            success: true,
            data: Some(user::UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                phone: user.phone,
                avatar: user.avatar,
                location: user.location,
                age: user.age,
                gender: user.gender,
            }),
            code: 200,
            message: None,
        },
        Err(_) => ResultModel {
            success: false,
            data: None,
            code: 404,
            message: Some("User doesn't exists.".to_string()),
        },
    }
}

pub async fn friends(identity: Identity, pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");
    match identity.identity() {
        Some(user_id_str) => {
            match web::block(move || {
                let self_user_id = user_id_str.parse::<i32>().unwrap();
                match schema::friends::dsl::friends
                    .filter(schema::friends::dsl::user_id.eq(&self_user_id))
                    .select(schema::friends::dsl::friend_user_id)
                    .inner_join(
                        schema::users::dsl::users
                            .on(schema::friends::dsl::friend_user_id.eq(schema::users::dsl::id)),
                    )
                    .select((
                        schema::users::dsl::id,
                        schema::users::dsl::username,
                        schema::users::dsl::email,
                        schema::users::dsl::phone,
                        schema::users::dsl::avatar,
                        schema::users::dsl::gender,
                        schema::users::dsl::age,
                        schema::users::dsl::location,
                    ))
                    .load::<(i32, String, String, String, String, i32, i32, String)>(&conn)
                {
                    Ok(friends_result) => Ok(ResultModel {
                        success: true,
                        data: Some(
                            friends_result
                                .iter()
                                .map(|item| {
                                    let f = item.clone();
                                    user::UserInfo {
                                        id: f.0,
                                        username: f.1,
                                        email: f.2,
                                        phone: f.3,
                                        avatar: f.4,
                                        gender: f.5,
                                        age: f.6,
                                        location: f.7,
                                    }
                                })
                                .collect::<Vec<_>>(),
                        ),
                        code: 200,
                        message: None,
                    }),
                    Err(_) => Ok(ResultModel {
                        success: true,
                        data: Some(Vec::<user::UserInfo>::default()),
                        code: 200,
                        message: None,
                    }),
                }
            })
            .await
            {
                Ok(result) => result,
                Err(BlockingError::Error(e)) => e,
                Err(BlockingError::Canceled) => ResultModel {
                    success: false,
                    data: None,
                    code: 500,
                    message: Some("Operation has been cancelled.".to_string()),
                },
            }
        }
        None => ResultModel {
            success: false,
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn add_friend(
    web::Path(user_id): web::Path<i32>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");

    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            if self_user_id == user_id {
                ResultModel::<String> {
                    success: false,
                    data: None,
                    code: 400,
                    message: Some("Cannot add yourself as friend.".to_string()),
                }
            } else {
                match web::block(move || {
                    match schema::friends::dsl::friends
                        .filter(
                            schema::friends::dsl::user_id
                                .eq(&self_user_id)
                                .and(schema::friends::dsl::friend_user_id.eq(&user_id)),
                        )
                        .select(schema::friends::dsl::friend_user_id)
                        .first::<i32>(&conn)
                    {
                        Ok(_) => Err(ResultModel::<String> {
                            success: false,
                            data: None,
                            code: 400,
                            message: Some("Already becomes friends.".to_string()),
                        }),
                        Err(_) => match conn.transaction(|| {
                            match (
                                diesel::insert_into(schema::friends::dsl::friends)
                                    .values((
                                        schema::friends::dsl::user_id.eq(&self_user_id),
                                        schema::friends::dsl::friend_user_id.eq(&user_id),
                                    ))
                                    .execute(&conn),
                                diesel::insert_into(schema::friends::dsl::friends)
                                    .values((
                                        schema::friends::dsl::user_id.eq(&user_id),
                                        schema::friends::dsl::friend_user_id.eq(&self_user_id),
                                    ))
                                    .execute(&conn),
                            ) {
                                (Ok(_), Ok(_)) => Ok(()),
                                _ => Err(diesel::result::Error::RollbackTransaction),
                            }
                        }) {
                            Ok(_) => Ok(ResultModel {
                                success: true,
                                data: None,
                                code: 200,
                                message: None,
                            }),
                            Err(_) => Err(ResultModel {
                                success: false,
                                data: None,
                                code: 500,
                                message: Some("Failed to add friends.".to_string()),
                            }),
                        },
                    }
                })
                .await
                {
                    Ok(result) => result,
                    Err(BlockingError::Error(e)) => e,
                    Err(BlockingError::Canceled) => ResultModel {
                        success: false,
                        code: 500,
                        data: None,
                        message: Some("Operation has been cancelled.".to_string()),
                    },
                }
            }
        }
        None => ResultModel {
            success: false,
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn delete_friend(
    web::Path(user_id): web::Path<i32>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");

    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                match schema::friends::dsl::friends
                    .filter(
                        schema::friends::dsl::user_id
                            .eq(&self_user_id)
                            .and(schema::friends::dsl::friend_user_id.eq(&user_id)),
                    )
                    .select(schema::friends::dsl::friend_user_id)
                    .first::<i32>(&conn)
                {
                    Ok(_) => match diesel::delete(
                        schema::friends::dsl::friends.filter(
                            schema::friends::dsl::user_id
                                .eq(&self_user_id)
                                .and(schema::friends::dsl::friend_user_id.eq(&user_id))
                                .or(schema::friends::dsl::user_id
                                    .eq(&user_id)
                                    .and(schema::friends::dsl::friend_user_id.eq(&self_user_id))),
                        ),
                    )
                    .execute(&conn)
                    {
                        Ok(_) => Ok(ResultModel {
                            success: true,
                            data: None,
                            code: 200,
                            message: None,
                        }),
                        Err(_) => Err(ResultModel {
                            success: false,
                            data: None,
                            code: 500,
                            message: Some("Failed to delete friends.".to_string()),
                        }),
                    },
                    Err(_) => Err(ResultModel::<String> {
                        success: false,
                        data: None,
                        code: 400,
                        message: Some("Hasn't become friends.".to_string()),
                    }),
                }
            })
            .await
            {
                Ok(result) => result,
                Err(BlockingError::Error(e)) => e,
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
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn update_profiles(
    web::Json(model): web::Json<UserInfoUpdateModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("Failed to get db connection from pool.");

    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                diesel::update(
                    schema::users::dsl::users.filter(schema::users::dsl::id.eq(&self_user_id)),
                )
                .set((
                    schema::users::dsl::username.eq(&model.username),
                    schema::users::dsl::email.eq(&model.email),
                    schema::users::dsl::phone.eq(&model.phone),
                    schema::users::dsl::location.eq(&model.location),
                    schema::users::dsl::age.eq(&model.age),
                    schema::users::dsl::gender.eq(&model.gender),
                    schema::users::dsl::avatar.eq(&model.avatar),
                ))
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
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}

pub async fn update_password(
    web::Json(model): web::Json<PasswordUpdateModel>,
    identity: Identity,
    pool: web::Data<DbPool>,
) -> impl Responder {
    if model.confirm_password != model.new_password {
        return ResultModel::<String> {
            success: false,
            data: None,
            code: 400,
            message: Some("Mismatch between new-password and confirm-password.".to_string()),
        };
    }

    let conn = pool.get().expect("Failed to get db connection from pool.");

    match identity.identity() {
        Some(user_id_str) => {
            let self_user_id = user_id_str.parse::<i32>().unwrap();
            match web::block(move || {
                match schema::users::dsl::users
                    .filter(schema::users::dsl::id.eq(&self_user_id))
                    .select(schema::users::dsl::password_hash)
                    .first::<String>(&conn)
                {
                    Ok(hash) => match bcrypt::verify(&model.original_password, &hash) {
                        Ok(true) => {
                            match diesel::update(
                                schema::users::dsl::users
                                    .filter(schema::users::dsl::id.eq(&self_user_id)),
                            )
                            .set(
                                schema::users::dsl::password_hash.eq(&bcrypt::hash(
                                    &model.new_password,
                                    bcrypt::DEFAULT_COST,
                                )
                                .unwrap()
                                .to_string()),
                            )
                            .execute(&conn)
                            {
                                Ok(_) => Ok(ResultModel::<String> {
                                    success: true,
                                    data: None,
                                    code: 200,
                                    message: None,
                                }),
                                Err(e) => Err(ResultModel::<String> {
                                    success: true,
                                    data: None,
                                    code: 500,
                                    message: Some(e.to_string()),
                                }),
                            }
                        }
                        _ => Err(ResultModel {
                            success: false,
                            data: None,
                            code: 401,
                            message: Some("Incorrect password.".to_string()),
                        }),
                    },
                    Err(_) => Err(ResultModel {
                        success: false,
                        data: None,
                        code: 401,
                        message: Some("Incorrect password.".to_string()),
                    }),
                }
            })
            .await
            {
                Ok(result) => result,
                Err(BlockingError::Error(e)) => e,
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
            data: None,
            code: 401,
            message: Some("Not logged in.".to_string()),
        },
    }
}
