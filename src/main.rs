use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, ResponseError};
use actix_web::cookie::{Cookie, time::Duration};
use actix_web::http::header;
use askama::Template;
use thiserror::Error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use rusqlite::Connection;
use serde::Deserialize;
// use base64;
//use base64::alphabet::STANDARD;
//use base64::engine::general_purpose::STANDARD;
use urlencoding::encode;
// use urlencoding::decode;
use regex::Regex;



#[derive(Template)]
#[template[path = "zubolite.html"]]
struct Zubolite {
    page: String, 
    title: String,
    sidemenu: String,
    body: String,    
}

#[derive(Template)]
#[template[path = "zubolite_input.html"]]
struct ZuboliteInput {
    page: String, 
    wtml: String,
    btn:  String,
    btn_label: String,    
}



#[derive(Debug, Error)]
enum PageError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),
    #[error("Failed to get DB connection")]
    ConnectionPoolError(#[from] r2d2::Error),
    #[error("Failed SQL execution")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for PageError{}


#[derive(Deserialize, Debug)]
struct PageParams {
    page: String,
    wtml: String,
    btn: String,
}

#[derive(Debug)]
struct PageData {
    id :u32,
    name:String,
    wtml:String,
    html:String,
}

/*
fn enc_base62(i:u32) -> Option<String> {
    let base_bytes = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".as_bytes();
    let mut bytes = [0x0, 0x0, 0x0, 0x0];
    let mut mi = i as usize;
    for d in 0..bytes.len() { // ４桁
        let rm = mi % base_bytes.len();
        mi = mi / base_bytes.len();
        bytes[d] = base_bytes[rm];
    }
    let r = String::from_utf8(bytes.to_vec());
    match r {
        Ok(s) => return Some(s),
        Err(_) => return None,
    }
}
    */
fn dec_base62(s:&String) -> u32 {
    let bytes = s.as_bytes();
    let mut r:u32 = 0;
    let mut dig:u32 = 1;
    let mut a:u32;
    if bytes.len() > 4 {
        return 0;
    }
    for d in 0..bytes.len() {
        let b = bytes[d];
        a = 0;
        if b'0' <= b && b <= b'9' {
            a = 52 + ((b - b'0') as u32);
        } else if b'A' <= b && b <= b'Z' {
            a = (b - b'A') as u32;        
        } else if b'a' <= b && b <= b'z' {
            a = 26 + ((b - b'a') as u32);        
        }
        r += dig * a;
        dig = dig * 62;
    }
    return r;
}

fn get_session_id(req: &HttpRequest) -> Option<String> {
    // クッキーを取得
    let cookies = req.cookies();
    // let mut session_id:String = String::from("test");
    match cookies {
        Ok(cookies) => {
            // cookies は Vec<Cookie> 型になっている
            //let my_cookie = cookies.get("my_cookie");
            let my_cookie = cookies.iter().find(|cookie| cookie.name() == "session_id");
            match my_cookie {
                Some(c) => {
                    //session_id = ;
                    return Some(c.value().to_string());
                },
                None => {
                    eprintln!("Error getting cookies:");
                    return None;
                },
            }
        },
        Err(err) => {
            // クッキー取得に失敗した場合の処理
            eprintln!("Error getting cookies: {}", err);
        },
    }
    return None;
}

fn select_from_name(conn: &Connection, page_name: &String
                ) -> Result<Vec<PageData>, PageError> {

    println!("select---1{}", &page_name);
    let mut stmt = conn.prepare("select id, name, wtml, html from page where name=?").expect("select失敗");
    let rows = stmt.query_map(&[&page_name], |row| {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let wtml = row.get(2)?;
        let html = row.get(3)?;
        Ok(PageData{id,name,wtml,html,})
    })?;

    let mut rvec = Vec::new();
    for row in rows {
        rvec.push(row?);
    }
    Ok(rvec)
}

fn esc_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

fn replace_first(text: &str, pattern: &str, replacement: &str) -> String {
    let re = Regex::new(pattern).unwrap();
    re.replace(text, replacement).to_string()
}

fn get_url(text:&str) -> Option<String> {
    // let re = Regex::new(r"^(.*?)").unwrap();
    //Regex::new(r"\((https?://\S+)\)")
    let re = Regex::new(r"\((\S+)\)").unwrap();

   // 文字列を正規表現で分割し、マッチした部分を取り出す
   let caps:Vec<_> = re.captures_iter(text)
               .map(|cap| cap[1].to_string())
                .collect();
    println!("caps={:?}", caps);
    if caps.len() == 1 {
        caps.get(0).cloned()
    } else {
        None
    }
}

fn get_line_html(text:&str) -> String {
    let re = Regex::new(r"\[(.*?)\]").unwrap();
    let vstr: Vec<&str> = re.split(text).collect();

    println!("lines={:?}", vstr);
   // 文字列を正規表現で分割し、マッチした部分を取り出す
   let caps:Vec<_> = re.captures_iter(text)
               .map(|cap| cap[1].to_string())
                .collect();
    println!("caps={:?}", caps);
    let mut html:String = String::from("");
    let mut i = 0;
    while i < vstr.len() {
        let mut img:bool = false;
        if let Some(s) = vstr.get(i) {
            let str = replace_first(s, r"\((\S+)\)", "");
            if str.ends_with('!') {
                let str = &s[..s.len() -1];
                html.push_str(&str);
                img = true;
            } else {
                html.push_str(&str);
            }
        }
        let mut next_str:&str = "";
        if i < vstr.len() - 1 {
            if let Some(s) = vstr.get(i + 1) {
                next_str = s;
            }
        }
        let url = get_url(next_str);
        let url_str = if let Some(u) = url {
            u
        } else {
            String::from("")
        };
        if i < caps.len() {
            if img == true {
                html.push_str("<img src='");
                html.push_str(&url_str);
                html.push_str("'");
                if let Some(c) = caps.get(i) {
                    html.push_str(" alt='");
                    html.push_str(c);
                    html.push_str("'");
                }
                html.push_str(" />");
            } else {
                if let Some(c) = caps.get(i) {
                    html.push_str("<a href='");
                    let enc = encode(c);
                    if url_str == "" {
                        html.push_str("./");
                        html.push_str(&enc);
                    } else {
                        html.push_str(&url_str);
                    }
                    html.push_str("'>");
                    html.push_str(c);        
                    html.push_str("</a>");
                }
            }
        }
        i = i + 1;
    }
    html
}


fn get_html_from_wtml(wtml:&String) -> String {
    let mut toc_html = String::from("<div class='table_of_contents'>\n");
    let mut html = String::from("");
    // 改行の処理
    let lines:Vec<&str> = wtml.lines().collect();
    let mut bm_index = 0;
    for line in lines {
        if line.starts_with("###-") {
            let line = line.trim_start_matches("###-");
            html.push_str(&format!("<h4>{}</h4>\n", &esc_html(&line)));
        } else if line.starts_with("###") {
            let line = line.trim_start_matches("###");
            bm_index = bm_index + 1;
            toc_html.push_str(&format!("　　　　　<a href='row{}'>{}</a><br/>\n", bm_index, &esc_html(&line)));
            html.push_str(&format!("<h4 id='row{}'>{}</h4>\n", bm_index, &esc_html(&line)));
        } else if line.starts_with("##-") {
            let line = line.trim_start_matches("##-");
            html.push_str(&format!("<h3>{}</h3>\n", &esc_html(&line)));
        } else if line.starts_with("##") {
            let line = line.trim_start_matches("##");
            bm_index = bm_index + 1;
            toc_html.push_str(&format!("　　　<a href='row{}'>{}</a><br/>\n", bm_index, &esc_html(&line)));
            html.push_str(&format!("<h3 id='row{}'>{}</32>\n", bm_index, &esc_html(&line)));
        } else if line.starts_with("#-") {
            let line = line.trim_start_matches("#-");
            html.push_str(&format!("<h2>{}</h2>\n", &esc_html(&line)));
        } else if line.starts_with("#") {
            let line = line.trim_start_matches("#");
            bm_index = bm_index + 1;
            toc_html.push_str(&format!("　<a href='row{}'>{}</a><br/>\n", bm_index, &esc_html(&line)));
            html.push_str(&format!("<h2 id='row{}'>{}</h2>\n", bm_index, &esc_html(&line)));
        } else if line.starts_with("---") {
            html.push_str("<hr />");
        } else {
            let line_html = get_line_html(&line);
            html.push_str(&line_html);
        }
        html.push_str("<br/>\n");
    }
    if bm_index > 0 {
        format!("{}</div>{}", toc_html, html)
    } else {
        html
    }
}


#[get("/{tail:.*}")]
async fn get_zubolite(tail: web::Path<String>,
                    req: HttpRequest,
                    db: web::Data<Pool<SqliteConnectionManager>>
                    ) -> Result<HttpResponse, PageError> {

    // クッキーからセッションIDを取得
    let session_id = get_session_id(&req);
    println!("セッションID:{:?}", session_id);


    let page_id = dec_base62(&tail.to_string());

    // ページ情報を取得
    let mut page_name = tail.into_inner();
    println!("page_name.len={}", page_name.len());
    if page_name.len() == 0 {
        page_name = "こんにちは".to_string();
    }

    let conn = db.get()?;




    println!("DB接続：{:?}", &conn);
    println!("get id == {:?}", page_id);

    let side_menu_name = "SideMenu".to_string();
    let side_menu_rows = select_from_name(&conn, &side_menu_name)?;
    println!("side_menu_rows:{:?}", &side_menu_rows);
    let side_html = if side_menu_rows.len() > 0 {
        side_menu_rows[0].html.to_string()
    } else {
        String::from ("")
    };

    let rows = select_from_name(&conn, &page_name)?;
    println!("rows:{:?}", &rows);

    let body_html:String ;
    if rows.len() == 0 {
        let zubo_input = ZuboliteInput {
            page: page_name.to_string(), 
            wtml: "wtml".to_string(),
            btn:  "insert".to_string(),
            btn_label: "登録".to_string(),
        };
        body_html = zubo_input.render()?;
    } else {
        let name = &rows[0].name;
        let wtml = &rows[0].wtml;
        body_html = rows[0].html.to_string();
        ZuboliteInput {
            page: name.to_string(), 
            wtml: wtml.to_string(),
            btn:  "insert".to_string(),
            btn_label: "登録".to_string(),
        };
    }
    let html = Zubolite{
     page: page_name.to_string(),
     title: page_name.to_string(),
     sidemenu: side_html.to_string(),
     body: body_html,
    };
    let response_body = html.render()?;

    let cookie = Cookie::build("session_id", "sessiontest")
       .path("/")
       .max_age(Duration::days(1))
       .secure(true) // HTTPS通信でのみ送る
       .http_only(true) // JavaScriptからアクセスできないようにする
       .finish();
    Ok(HttpResponse::Ok().cookie(cookie).body(response_body))
}

#[post("/{tail:.*}")]
async fn post_zubolite(tail: web::Path<String>, 
                    params: web::Form<PageParams>,
                    db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, PageError> {
    let cookie = Cookie::build("my_cookie", "value")
        .path("/")
        .max_age(Duration::days(1))
        .secure(true) // HTTPS通信でのみ送る
        .http_only(true) // JavaScriptからアクセスできないようにする
        .finish();
    let vstr = tail.to_string();
    let v0 = dec_base62(&vstr);
    println!("tail == {:?}", tail.into_inner());
    println!("id == {:?}", v0);
    println!("post page == {:?}", params.page);
    println!("post wtml == {:?}", params.wtml);
    println!("post btn == {:?}", params.btn);
    if params.btn == "go" {
        //画面遷移
        let encoded = encode(params.page.as_str());
        let location = format!("./{}", &encoded);
        let mut res = HttpResponse::SeeOther();
        res.append_header((header::LOCATION, location));   
        return Ok(res.finish());
    }
    if params.btn == "insert" {
        let conn = db.get()?;
        let wtml = &params.wtml;
        let html = get_html_from_wtml(&wtml);
        conn.execute("
            insert into page(name, wtml, html) values(?, ?, ?)
            ", &[&params.page, &params.wtml, &html])?;
        
        //画面遷移
        let encoded = encode(params.page.as_str());

        let location = format!("./{}", &encoded);
        let mut res = HttpResponse::SeeOther();
        res.append_header((header::LOCATION, location));   
        return Ok(res.finish());
    }
    if params.btn == "update" {
        println!("update");
        let conn = db.get()?;
        let wtml = &params.wtml;
        let html = get_html_from_wtml(&wtml);
        
        
        conn.execute("
            update page set wtml=?, html=? where name=?
            ", &[&params.wtml, &html, &params.page]).expect("登録エラー");
        
        //画面遷移
        let encoded = encode(params.page.as_str());
        let location = format!("./{}", &encoded);
        let mut res = HttpResponse::SeeOther();
        res.append_header((header::LOCATION, location));   
        return Ok(res.finish());
    }
    let conn = db.get()?;
    let page_name = &params.page;
    println!("DB接続：{:?}", &conn);
    let rows = select_from_name(&conn, &page_name)?;
    let zubo_input:ZuboliteInput;
    if rows.len() == 0 {
        zubo_input = ZuboliteInput {
            page: params.page.to_string(), 
            wtml: "wtml".to_string(),
            btn:  "insert".to_string(),
            btn_label: "登録".to_string(),
        };
    } else{
        let name = &rows[0].name;
        let wtml = &rows[0].wtml;
        zubo_input = ZuboliteInput {
            page: name.to_string(), 
            wtml: wtml.to_string(),
            btn:  "update".to_string(),
            btn_label: "更新".to_string(),
        };
    }
    let side_menu_name = "SideMenu".to_string();
    let side_menu_rows = select_from_name(&conn, &side_menu_name)?;
    println!("side_menu_rows:{:?}", &side_menu_rows);
    let side_html = if side_menu_rows.len() > 0 {
        side_menu_rows[0].html.to_string()
    } else {
        String::from ("")
    };

    let input_html = zubo_input.render()?;
    let html = Zubolite{
     page: page_name.to_string(),
     title: page_name.to_string(),
     sidemenu: side_html.to_string(),
     body: input_html,
    };
    let response_body = html.render()?;
    return Ok(HttpResponse::Ok().cookie(cookie).body(response_body));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("zubolite.db");
    let pool = Pool::new(manager).expect("接続プール初期化失敗！");
    let conn = pool.get().expect("接続取得失敗！");
    conn.execute("
        create table if not exists page (
            id integer primary key autoincrement,
            name TEXT NOT NULL UNIQUE,
            wtml TEXT NOT NULL,
            html TEXT NOT NULL
        )
    ", params![]).expect("テーブル作成失敗！");
    HttpServer::new(move|| {App::new()
                    .service(get_zubolite)
                    .service(post_zubolite)
                    .app_data(web::Data::new(pool.clone()))})
        .bind(("0.0.0.0", 1582))?
        .run()
        .await?;
    Ok(())
}


