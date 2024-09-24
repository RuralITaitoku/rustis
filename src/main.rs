use actix_web::{get, web, App, HttpResponse, HttpServer, ResponseError};
use actix_web::cookie::{Cookie, time::Duration};
use askama::Template;
use thiserror::Error;

struct TodoEntry {
    id: u32,
    text: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>, 
}

#[derive(Debug, Error)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),
}

impl ResponseError for MyError{}

#[get("/{tail:.*}")]
async fn index(tail: web::Path<String>) -> Result<HttpResponse, MyError> {
    let mut entries = Vec::new();
    entries.push(TodoEntry{
        id: 1,
        text: "最初のテキスト".to_string(),
    });
    entries.push(TodoEntry{
        id: 2,
        text: "次のテキスト".to_string(),
    });
    let html = IndexTemplate{ entries };
    let response_body = html.render()?;

    println!("tail == {:?}", tail);

    let cookie = Cookie::build("my_cookie", "value")
       .path("/")
       .max_age(Duration::days(1))
       .secure(true) // HTTPS通信でのみ送る
       .http_only(true) // JavaScriptからアクセスできないようにする
       .finish();

    Ok(HttpResponse::Ok().cookie(cookie).body(response_body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
        HttpServer::new(move|| {App::new().service(index)})
        .bind(("0.0.0.0", 1582))?
        .run()
        .await
}