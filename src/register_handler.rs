use crate::database::Database;
use crate::models::JsonUser;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

pub async fn register_user(
    database: web::Data<Arc<Database>>,
    user_data: web::Json<JsonUser>,
) -> HttpResponse {
    match database.contains_user(user_data.login.as_bytes()) {
        Ok(true) => HttpResponse::Conflict(),
        Err(_) => HttpResponse::InternalServerError(),
        _ => match database.add_user(user_data.login.as_bytes(), user_data.password.as_bytes()) {
            Ok(_) => HttpResponse::Ok(),
            Err(_) => HttpResponse::InternalServerError(),
        },
    }
    .finish()
}
