mod controllers;
mod models;

use crate::controllers::__path_update_user;
use crate::controllers::__path_get_user;
use crate::controllers::__path_delete_user;
use crate::controllers::__path_list_users;
use crate::controllers::__path_create_user;

use actix_web::{web, App, HttpServer, middleware::Logger};
use controllers::{create_user, list_users, get_user, update_user, delete_user};
use dotenv::dotenv;
use sea_orm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        list_users,
        get_user,
        update_user,
        delete_user
    ),
    components(
        schemas(crate::models::Model, crate::controllers::CreateUser)
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await.expect("Failed to connect to database");

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .route("/users", web::post().to(create_user))
            .route("/users", web::get().to(list_users))
            .route("/users/{id}", web::delete().to(delete_user))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{id}", web::put().to(update_user))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}