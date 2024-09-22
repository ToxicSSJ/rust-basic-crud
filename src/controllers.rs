use actix_web::{web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel, ActiveModelTrait, Set};
use crate::models::{self, Entity as User};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate};

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, max = 50))]
    pub name: String,

    #[validate(email, length(max = 100))]
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = models::Model),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_user(
    db: web::Data<DatabaseConnection>,
    new_user: web::Json<CreateUser>,
) -> impl Responder {

    if let Err(validation_errors) = new_user.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    let user = models::ActiveModel {
        name: Set(new_user.name.to_string()),
        email: Set(new_user.email.to_string()),
        ..Default::default()
    };

    match User::insert(user).exec(db.get_ref()).await {
        Ok(insert_result) => {
            let inserted_user = User::find_by_id(insert_result.last_insert_id).one(db.get_ref()).await;
            match inserted_user {
                Ok(Some(user_model)) => HttpResponse::Created().json(user_model),
                Ok(None) => HttpResponse::InternalServerError().body("Error retrieving created user"),
                Err(_) => HttpResponse::InternalServerError().body("Error retrieving created user"),
            }

        },
        Err(_) => HttpResponse::BadRequest().body("Error creating user"),
    }
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (status = 200, description = "User retrieved successfully", body = models::Model),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user(
    db: web::Data<DatabaseConnection>,
    user_id: web::Path<i32>,
) -> impl Responder {
    match User::find_by_id(user_id.into_inner()).one(db.get_ref()).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error retrieving user"),
    }
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = [models::Model])
    )
)]
pub async fn list_users(db: web::Data<DatabaseConnection>) -> impl Responder {
    let users = User::find().all(db.get_ref()).await.unwrap();
    HttpResponse::Ok().json(users)
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User updated successfully", body = models::Model),
        (status = 404, description = "User not found"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn update_user(
    db: web::Data<DatabaseConnection>,
    user_id: web::Path<i32>,
    updated_user: web::Json<CreateUser>,
) -> impl Responder {

    if let Err(validation_errors) = updated_user.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    match User::find_by_id(user_id.into_inner()).one(db.get_ref()).await {
        Ok(Some(mut user)) => {
            let mut user_model = user.into_active_model();
            user_model.name = Set(updated_user.name.to_string());
            user_model.email = Set(updated_user.email.to_string());
            match user_model.update(db.get_ref()).await {
                Ok(updated_user) => HttpResponse::Ok().json(updated_user),
                Err(_) => HttpResponse::InternalServerError().body("Error updating user"),
            }
        },
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error retrieving user"),
    }
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 404, description = "User not found")
    )
)]
pub async fn delete_user(
    db: web::Data<DatabaseConnection>,
    user_id: web::Path<i32>,
) -> impl Responder {
    match User::delete_by_id(user_id.into_inner()).exec(db.get_ref()).await {
        Ok(result) => {
            if result.rows_affected > 0 {
                HttpResponse::Ok().body("User deleted")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        }
        Err(_) => HttpResponse::BadRequest().body("Error deleting user"),
    }
}