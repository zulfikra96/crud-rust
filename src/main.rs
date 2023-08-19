
use dotenv::dotenv;
use actix_web::{  get, post, put, web, App, HttpResponse, HttpServer, Responder, Error, delete };
use tokio_postgres::{NoTls, GenericClient,};
use serde::{Deserialize, Serialize};
use deadpool_postgres::{Client, Pool, Manager, ManagerConfig, RecyclingMethod};
use chrono::{  NaiveDateTime };

struct AppState {
    app_name: String
}

#[derive(Deserialize, Serialize)]
struct Users  {
    id: i32,
    name: String,
    created_at: Option<NaiveDateTime>
}

#[derive(Deserialize, Serialize)] 
struct UsersRequest {
    name: String
}

#[derive(Deserialize, Serialize)]
enum Status {
    SUCCESS,
    FAIL
}

#[derive(Deserialize, Serialize)] 
struct Response {
    status: String,
    status_code: u32
}

#[derive(Debug, Default, Deserialize)]
struct ExampleConfig{
    server_addr: String,
    pg: deadpool_postgres::Config
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}

#[get("/state")]
async fn getState(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[get("/users")]
async fn get_users(db_pool: web::Data<Pool>) -> impl Responder {
    let client: Client = db_pool.get().await.unwrap();
    let rows = client.query("SELECT id, name, created_at FROM users", &[]).await
        .unwrap();
        let mut wrap: Vec<Users> = Vec::new();
        for e in rows.iter()  {
            wrap.push( Users {
                id: e.get(0),
                name: e.get(1),
                created_at: e.get(2)
            });
        }
        HttpResponse::Ok().json(wrap)
}

#[post("/users")]
async fn create_users(request_body: web::Json<UsersRequest>, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let client = db_pool.get().await.unwrap();
    client.execute("INSERT INTO users (name) VALUES($1)", &[&request_body.name]).await.unwrap();
    let _response = Response { status: String::from("success"), status_code: 422 };
    Ok(HttpResponse::Ok().json(_response))
}

#[put("/users/{user_id}")]
async fn update_users(request_body: web::Json<UsersRequest>, db_pool: web::Data<Pool>, path: web::Path<i32>) -> Result<HttpResponse, Error> {
    let(user_id) = path.into_inner();

    let client = db_pool.get().await.unwrap();
    client.execute("UPDATE users SET name = $1 WHERE id = $2", &[&request_body.name, &user_id]).await.unwrap();
    let _response = Response { status: String::from("success"), status_code: 422 };
    Ok(HttpResponse::Ok().json(_response))
}

#[delete("/users/{user_id}")]
async fn delete_users(path: web::Path<i32>, db_pool: web::Data<Pool>) -> Result<HttpResponse, Error>  {
    let(user_id) = path.into_inner();
    let client = db_pool.get().await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
    let _response = Response { status: String::from("success"), status_code: 422 };
    Ok(HttpResponse::Ok().json(_response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host("localhost");
    pg_config.port(5678);
    pg_config.user("crud");
    pg_config.dbname("crud");
    pg_config.password("crud");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();
    HttpServer::new(move || {
        App::new()
            .service(hello)
            .app_data(web::Data::new(AppState {
                app_name: String::from("Hello actix web")
            }))
            .service(getState)
            .app_data(web::Data::new(pool.clone()))
            .service(get_users)
            .service(update_users)
            .service(delete_users)
    })
    .bind(("127.0.0.1", 8080)).unwrap()
    .run()
    .await
}