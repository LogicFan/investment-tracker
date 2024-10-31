use actix_files::Files;
use actix_web::{web, App, HttpServer};
use flexfolio::{investment, user};
// use server::{auth, constant, investment};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    if let Err(error) = flexfolio::init() {
        panic!("Fail to initialize the server with error: {}", error)
    }

    HttpServer::new(move || {
        App::new()
            .service(user::register::handler)
            .service(user::login::handler)
            .service(user::rotate::handler)
            .service(user::update::handler)
            .service(user::delete::handler)
            .service(user::exist::handler)
            .service(investment::account::insert::handler)
            .service(investment::account::fetch::handler)
            .service(investment::account::update::handler)
            .service(investment::account::delete::handler)
            // .service(investment::account::delete)
            .service(Files::new("/", "dist/").index_file("index.html"))
            .default_service(web::to(flexfolio::index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
