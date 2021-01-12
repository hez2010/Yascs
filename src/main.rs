#[macro_use]
extern crate diesel;
extern crate chrono;

mod api;
mod model;
mod schema;

use actix::Actor;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web, App, HttpResponse, HttpServer};
use api::{message, user};
use diesel::{r2d2, r2d2::ConnectionManager, PgConnection};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("No connection string specified in environment variable DATABASE_URL.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let stream = message::MessageStreamServer::new().start();
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(stream.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("mosad_user")
                    .http_only(true)
                    .secure(false),
            ))
            .service(
                web::scope("/api")
                    .service(web::scope("/user").configure(user::config))
                    .service(web::scope("/message").configure(message::config)),
            )
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Welcome to MOSAD Group 11 Backend!")),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
