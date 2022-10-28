use crate::blog::BlogPost;
use crate::errors::ActixError;
use crate::Blog;
use actix_web::http::header::ContentType;
use actix_web::http::Method;
use actix_web::{get, web, HttpResponse};
use sailfish::TemplateOnce;
use std::sync::Arc;

trait ContentResponse: TemplateOnce {
    fn to_response(self) -> HttpResponse {
        match self.render_once() {
            Ok(b) => HttpResponse::Ok().content_type(ContentType::html()).body(b),
            Err(err) => HttpResponse::from_error(ActixError(err)),
        }
    }
}

// Sailfish (EJS syntax) does not support "extend" templates,
// so each function execute another struct
#[derive(TemplateOnce)]
#[template(path = "articles.html")]
struct Articles<'a> {
    articles: &'a [(String, BlogPost)],
}

impl<'a> ContentResponse for Articles<'a> {}

#[derive(TemplateOnce)]
#[template(path = "content.html")]
struct Message {
    content: &'static str,
}

impl ContentResponse for Message {}

pub async fn default_handler(req_method: Method) -> HttpResponse {
    match req_method {
        Method::GET => HttpResponse::Found()
            .insert_header(("Location", "/404"))
            .finish(),
        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}

pub async fn home_page(
    blog: web::Data<Arc<Blog>>,
    params: Option<web::Path<usize>>,
) -> HttpResponse {
    let i = match params {
        Some(i) => *i,
        None => 1,
    };

    let articles = if blog.size() < i * 10 - 10 {
        return HttpResponse::Found()
            .insert_header(("Location", "/404"))
            .finish();
    } else if blog.size() < i * 10 - 1 {
        blog.take_articles(i * 10 - 10, blog.size())
    } else {
        blog.take_articles(i * 10 - 10, i * 10)
    };

    let content = Articles { articles };

    content.to_response()
}

#[get("/404")]
pub async fn not_found() -> HttpResponse {
    let content = Message {
        content: "This page does not exist!",
    };

    content.to_response()
}
