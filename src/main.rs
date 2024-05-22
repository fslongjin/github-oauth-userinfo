use std::{collections::HashMap, str::FromStr};

use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};

use reqwest::header::{HeaderName, HeaderValue};
use simple_logger::SimpleLogger;

const GITHUB_API_URL: &str = "https://api.github.com";

const GITHUB_USER: &str = "/user";
const GITHUB_USER_EMAILS: &str = "/user/emails";
const GITHUB_USER_PUBLIC_EMAILS: &str = "/user/public_emails";

async fn get_user_info(
    headers: &reqwest::header::HeaderMap,
) -> Result<HashMap<String, serde_json::Value>, reqwest::Error> {
    let mut headers = headers.clone();
    headers.insert(
        HeaderName::from_bytes("Accept".as_bytes()).unwrap(),
        HeaderValue::from_bytes("application/vnd.github.v3+json".as_bytes()).unwrap(),
    );

    // info!("get_user_info: headers: {:?}", headers);

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}{}", GITHUB_API_URL, GITHUB_USER))
        .headers(headers.clone())
        .send()
        .await?;
    // info!("get_user_info: response: {:?}", response);
    // // 把返回的结果转换成json格式
    let text = response.text().await.expect("text error");
    let result: HashMap<String, serde_json::Value> =
        serde_json::from_str(&text).expect("json error");
    Ok(result)
}

async fn get_user_email(
    headers: &reqwest::header::HeaderMap,
    public: bool,
) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();

    let posifix = if public {
        GITHUB_USER_PUBLIC_EMAILS
    } else {
        GITHUB_USER_EMAILS
    };
    let response = client
        .get(&format!("{}{}", GITHUB_API_URL, posifix))
        .headers(headers.clone())
        .send()
        .await?;

    // 把返回的结果转换成json列表格式
    let json: serde_json::Value = response.json().await?;
    let emails = json.as_array().unwrap_or(&vec![]).clone();

    let mut verified_email = None;

    for email in emails.iter() {
        if email["verified"].as_bool().unwrap_or(false) {
            if email["primary"].as_bool().unwrap_or(false) {
                return Ok(Some(email["email"].as_str().unwrap_or("").to_string()));
            } else if verified_email.is_none() {
                if let Some(email) = email["email"].as_str() {
                    verified_email = Some(email.to_string());
                }
            }
        }
    }

    Ok(verified_email)
}

fn to_reqwest_headers(headers: &actix_web::http::header::HeaderMap) -> reqwest::header::HeaderMap {
    let mut reqwest_headers = reqwest::header::HeaderMap::new();

    for (key, value) in headers.iter() {
        if key.as_str().to_ascii_lowercase() != "authorization" {
            continue;
        }

        reqwest_headers.insert(
            HeaderName::from_str(key.as_str()).unwrap(),
            HeaderValue::from_bytes(value.as_bytes()).unwrap(),
        );
    }

    reqwest_headers.insert(
        HeaderName::from_bytes("X-GitHub-Api-Version".as_bytes()).unwrap(),
        HeaderValue::from_bytes(b"2022-11-28").unwrap(),
    );
    reqwest_headers.insert(
        HeaderName::from_bytes("host".as_bytes()).unwrap(),
        HeaderValue::from_bytes(b"api.github.com").unwrap(),
    );
    reqwest_headers.insert(
        HeaderName::from_bytes("user-agent".as_bytes()).unwrap(),
        HeaderValue::from_bytes(b"Go-http-client/1.1").unwrap(),
    );

    reqwest_headers
}

#[get("/api/user")]
async fn userinfo(req: HttpRequest) -> impl Responder {
    // info!("initial headers: {:?}", req.headers());
    let headers = to_reqwest_headers(req.headers());
    // info!("headers: {:?}", headers);
    let mut user_info = get_user_info(&headers).await.unwrap();

    // info!("user_info: {:?}", user_info);

    if !user_info.contains_key("email")
        || user_info["email"].is_null()
        || user_info["email"].as_str().unwrap().is_empty()
    {
        // 需要获取用户的邮箱信息

        if let Ok(Some(email)) = get_user_email(&headers, true).await {
            user_info.insert("email".to_string(), serde_json::Value::String(email));
        } else if let Ok(Some(email)) = get_user_email(&headers, false).await {
            user_info.insert("email".to_string(), serde_json::Value::String(email));
        }
    }

    // info!("user_info: {:?}", user_info);
    HttpResponse::Ok().json(user_info)
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world! This is github oauth user info service.\n")
}

fn logger_init() {
    // 初始化日志系统，日志级别为Info
    // 如果需要调试，可以将日志级别设置为Debug
    let logger = SimpleLogger::new().with_level(log::LevelFilter::Info);

    logger.init().unwrap();
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger_init();
    HttpServer::new(|| App::new().service(userinfo).service(index))
        .bind(("0.0.0.0", 10001))?
        .run()
        .await
}
