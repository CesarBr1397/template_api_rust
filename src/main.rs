mod models;
mod db;
mod response;

use actix_web::{test::status_service, web, App, HttpServer, Responder};
use models::{User, CreateUser, UpdateUser, DeleteUser};
use response::{AppError, AppResponse, OkModel};
use serde::de::value::Error;
use sqlx::PgPool;
use std::sync::OnceLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        get_user,
        create_user,
        update_user,
        delete_user
    ),
    components(
        schemas(
            User,
            CreateUser,
            UpdateUser,
            DeleteUser
        )
    ),
    tags(
        (name = "Users", description = "API de usuarios")
    )
)]
struct ApiDoc;

static OPENAPI: OnceLock<utoipa::openapi::OpenApi> = OnceLock::new();

// Obtener todos los usuarios
#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    responses(
        (status = 200, body = Vec<User>, description = "List of users"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_users(pool: web::Data<PgPool>) -> Result<web::Json<OkModel<Vec<User>>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(pool.get_ref())
        .await?;  // El operador ? convierte automáticamente sqlx::Error a AppError
        
    Ok(web::Json(OkModel {
        success: true,
        data: users,
    }))
}

// Obtener un usuario por ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    responses(
        (status = 200, body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, description = "User ID")
    )
)]
async fn get_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> Result<web::Json<OkModel<User>>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = $1")
        .bind(user_id.into_inner())
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::Invalid { err: "Usuario no encontrado" },
            _ => {
                log::error!("Error de base de datos: {}", e);
                AppError::InternalError
            }
        })?;

    Ok(web::Json(OkModel {
        success: true,
        data: user,
    }))
}

// Crear un usuario
#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    request_body = CreateUser,
    responses(
        (status = 201, body = User),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create_user(
    pool: web::Data<PgPool>,
    new_user: web::Json<CreateUser>,
) -> Result<web::Json<OkModel<User>>, AppError> {
    // 1. Validación básica del input
    if new_user.name.is_empty() || new_user.email.is_empty() {
        return Err(AppError::Invalid {
            err: "Nombre y email son requeridos",
        });
    }

    // 2. Validación de formato de email (ejemplo simple)
    if !new_user.email.contains('@') {
        return Err(AppError::Invalid {
            err: "Formato de email inválido",
        });
    }

    // 3. Ejecutar la consulta con manejo de errores
    match sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email"
    )
    .bind(&new_user.name)
    .bind(&new_user.email)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(user) => Ok(web::Json(OkModel {
            success: true,
            data: user,
        })),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            // Violación de constraint UNIQUE (email duplicado)
            Err(AppError::Invalid {
                err: "El email ya está registrado",
            })
        }
        Err(e) => {
            // Registrar error inesperado
            log::error!("Error al crear usuario: {}", e);
            Err(AppError::InternalError)
        }
    }
}

// Actualizar un usuario
#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "Users",
    request_body = CreateUser,
    responses(
        (status = 200, body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, description = "User ID")
    )
)]
async fn update_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    updated_user: web::Json<CreateUser>,
) -> Result<web::Json<OkModel<User>>, AppError> {
    let user_id = user_id.into_inner();

    // 1. Validación de los datos de entrada
    if updated_user.name.is_empty() || updated_user.email.is_empty() {
        return Err(AppError::Invalid {
            err: "Nombre y email son requeridos",
        });
    }

    // 2. Validación básica de formato de email
    if !updated_user.email.contains('@') {
        return Err(AppError::Invalid {
            err: "Formato de email inválido",
        });
    }

    // 3. Ejecutar la actualización con manejo de errores
    match sqlx::query_as::<_, User>(
        "UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING id, name, email"
    )
    .bind(&updated_user.name)
    .bind(&updated_user.email)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(user) => Ok(web::Json(OkModel {
            success: true,
            data: user,
        })),
        Err(sqlx::Error::RowNotFound) => {
            // Usuario no encontrado
            Err(AppError::Invalid {
                err: "Usuario no encontrado",
            })
        },
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            // Email ya existe
            Err(AppError::Invalid {
                err: "El email ya está registrado por otro usuario",
            })
        },
        Err(e) => {
            // Error inesperado de base de datos
            log::error!("Error al actualizar usuario {}: {}", user_id, e);
            Err(AppError::InternalError)
        }
    }
}

// Eliminar un usuario
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, description = "User ID")
    )
)]
async fn delete_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> Result<web::Json<OkModel<()>>, AppError> {
    let user_id = user_id.into_inner();
    
    match sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(web::Json(OkModel {
                success: true,
                data: (),
            }))
        },
        Ok(_) => {
            // No rows affected - user didn't exist
            Err(AppError::Invalid {
                err: "Usuario no encontrado",
            })
        },
        Err(e) => {
            log::error!("Error al eliminar usuario {}: {}", user_id, e);
            Err(AppError::InternalError)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::get_db_pool().await.unwrap();
    
    // Initialize OpenAPI documentation
    let openapi = OPENAPI.get_or_init(|| ApiDoc::openapi());

    // Asigna el HttpServer a la variable server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::resource("/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(create_user)),
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(get_user))
                    .route(web::put().to(update_user))
                    .route(web::delete().to(delete_user)),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind("127.0.0.1:8080")?;

    // URL de Swagger UI
    let swagger_url = "http://localhost:8080/swagger-ui/";

    println!("Servidor iniciado en {}", swagger_url);

    // Intenta abrir el navegador
    if webbrowser::open(swagger_url).is_err() {
        println!("No se pudo abrir el navegador automáticamente. Por favor visita: {}", swagger_url);
    }

    // Inicia el servidor
    server.run().await

}
