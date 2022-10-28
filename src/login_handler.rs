use crate::database::Database;
use crate::models::JsonUser;
use crate::password::Password;
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use std::sync::Arc;

pub async fn login_user(
    req: HttpRequest,
    user_data: web::Json<JsonUser>,
    database: web::Data<Arc<Database>>,
) -> HttpResponse {
    let user = match database.get_user(user_data.login.as_bytes()) {
        Ok(h) => h,
        _ => return HttpResponse::Unauthorized().finish(),
    };

    let hashed = match argon2::PasswordHash::new(&user.hash) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let password = Password::new();

    match password.verify_password(user_data.password.as_bytes(), hashed) {
        Ok(_) => {
            if Identity::login(&req.extensions(), user_data.login.clone()).is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
        Err(e) => match e {
            argon2::password_hash::Error::Password => HttpResponse::Unauthorized().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}
