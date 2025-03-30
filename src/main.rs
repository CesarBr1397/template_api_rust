mod models;
mod db;
mod response;
use actix_web::{web, App, HttpServer, Responder};
use models::{User, CreateUser};
use response::AppResponse;
use sqlx::PgPool;

// Obtener todos los usuarios
async fn get_users(pool: web::Data<PgPool>) -> impl Responder {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();
    AppResponse::Success(users).response()
}

// Obtener un usuario por ID
async fn get_user(pool: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = $1")
        .bind(user_id.into_inner())
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    AppResponse::Success(user).response()
}

// Crear un usuario
async fn create_user(pool: web::Data<PgPool>, new_user: web::Json<CreateUser>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email")
        .bind(&new_user.name)
        .bind(&new_user.email)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    AppResponse::Success(user).response()
}

// Actualizar un usuario
async fn update_user(pool: web::Data<PgPool>, user_id: web::Path<i32>, updated_user: web::Json<CreateUser>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING id, name, email")
        .bind(&updated_user.name)
        .bind(&updated_user.email)
        .bind(user_id.into_inner())
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    AppResponse::Success(user).response()
}

// Eliminar un usuario
async fn delete_user(pool: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id.into_inner())
        .execute(pool.get_ref())
        .await
        .unwrap();
    AppResponse::Success(()).response()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::get_db_pool().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::get().to(get_users))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}