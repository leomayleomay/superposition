mod api;
mod models;
mod db;
mod messages;
mod handlers;
use api::global_config::{
    get_global_config_key,
    get_config,
    post_config_key_value,
};

use api::dimensions::{
    get_dimensions,
    get_dimension_key,
    post_dimension
    
};

use diesel::{
    r2d2::{ConnectionManager,Pool},
    PgConnection
};
use dotenv;
use std::env;

use db::utils::{get_pool, AppState, DbActor};
use actix::SyncArbiter;
use actix_web::{HttpServer, App, web::scope, middleware::Logger,web::Data};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");
    let pool: Pool<ConnectionManager<PgConnection>> = get_pool(&db_url);
    let db_addr = SyncArbiter::start(5, move || DbActor(pool.clone()));
    HttpServer::new(move || {
        let logger: Logger = Logger::default();
        App::new()
        .app_data(Data::new(AppState {db: db_addr.clone()}))
        .wrap(logger)
        .service(
            scope("/global_config")
                .service(get_config)
                .service(get_global_config_key)
                .service(post_config_key_value)
        )
        .service(
            scope("/dimensions")
                .service(get_dimensions)
                .service(get_dimension_key)
                .service(post_dimension)
        )
    })
    .bind(("127.0.0.1", 8000))?
    .workers(5)
    .run()
    .await
}
