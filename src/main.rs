mod models;
mod db;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use models::{User, CreateUser};
use sqlx::PgPool;

// Obtener todos los usuarios
async fn get_users(pool: web::Data<PgPool>) -> impl Responder {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(users)
}

// Obtener un usuario por ID
async fn get_user(pool: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM get_user_by_id($1)")
        .bind(user_id.into_inner())
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().json(user)
}

// Crear un usuario
async fn create_user(pool: web::Data<PgPool>, user: web::Json<CreateUser>) -> impl Responder {
    let new_user = sqlx::query_as::<_, User>(
        "SELECT * FROM create_user($1, $2)",
    )
    .bind(&user.name)
    .bind(&user.email)
    .fetch_one(pool.get_ref())
    .await
    .unwrap();
    HttpResponse::Created().json(new_user)
}

// Actualizar un usuario
async fn update_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    user: web::Json<CreateUser>,
) -> impl Responder {
    let updated_user = sqlx::query_as::<_, User>(
        "SELECT * FROM update_user($1, $2, $3)",
    )
    .bind(user_id.into_inner())
    .bind(&user.name)
    .bind(&user.email)
    .fetch_one(pool.get_ref())
    .await
    .unwrap();
    HttpResponse::Ok().json(updated_user)
}

// Eliminar un usuario
async fn delete_user(pool: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let deleted_user = sqlx::query_as::<_, User>(
        "SELECT * FROM delete_user($1)",
    )
    .bind(user_id.into_inner())
    .fetch_one(pool.get_ref())
    .await
    .unwrap();
    HttpResponse::Ok().json(deleted_user)
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