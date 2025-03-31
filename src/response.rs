use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    web, HttpResponse, Result,
};
use derive_more::{Display, Error}; // Para implementar automáticamente `Display` y `Error`
use log::warn;
use serde::Serialize;


/// Tipo de resultado estándar usado por los controladores (handlers).
pub type AppResult<T> = actix_web::Result<web::Json<OkModel<T>>, AppError>;

/// Modelo de respuesta para errores.
#[derive(Serialize)]
pub struct ErrModel {
    pub success: bool,
    pub err: &'static str,
}

/// Modelo de respuesta para éxitos.
#[derive(Serialize)]
pub struct OkModel<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
}

/// `AppError` representa los errores que pueden ocurrir en la aplicación.
#[derive(Debug, Display, Error, Serialize)]
pub enum AppError {
    /// Error por solicitud inválida (400)
    Invalid { err: &'static str },
    /// Error interno del servidor (500)
    InternalError,
}

/// Implementación para convertir `AppError` en una respuesta HTTP.
impl error::ResponseError for AppError {
    /// Devuelve el código de estado HTTP correspondiente al error.
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::Invalid { .. } => StatusCode::BAD_REQUEST, // 400
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }

    /// Genera la respuesta HTTP correspondiente al error.
    fn error_response(&self) -> HttpResponse {
        let mut builder = HttpResponse::build(self.status_code());
        let resp = builder.insert_header(ContentType::json());

        match *self {
            // Error de cliente (400)
            Self::Invalid { err } => resp.json(ErrModel {
                success: false,
                err,
            }),
            // Error de servidor (500), mensaje oculto al cliente
            Self::InternalError => resp.json(ErrModel {
                success: false,
                err: "500 error interno del servidor",
            }),
        }
    }
}

/// Convierte un error de SQLx en un `AppError` tipo interno
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        warn!("{}", err); // Se registra en logs
        Self::InternalError
    }
}


/// Enum que encapsula distintos tipos de respuesta de la aplicación.

/// `T` es el tipo de dato que se devolverá en caso de éxito.
#[derive(Serialize, Debug, Display)]
pub enum AppResponse<T>
where
    T: Serialize,
{
    /// Respuesta exitosa (200 OK)
    Success(T),
    /// Solicitud inválida (400 Bad Request)
    Invalid(&'static str),
    /// Error interno del servidor (500)
    
    /// ⚠️ El mensaje no se envía al cliente, pero sí se registra en los logs.
    InternalError(&'static str),
}

impl<T> AppResponse<T>
where
    T: Serialize,
{
    /// Método que genera la respuesta real que se enviará al cliente.
    
    /// Ejemplos de uso:
    /// - `AppResponse::Success(...)`
    /// - `AppResponse::Invalid(...)`
    /// - `AppResponse::InternalError(...)`
    pub fn response(self) -> Result<web::Json<OkModel<T>>, AppError> {
        match self {
            Self::Success(data) => Ok(web::Json(OkModel {
                success: true,
                data,
            })),
            Self::Invalid(err) => Err(AppError::Invalid { err }),
            Self::InternalError(err) => {
                warn!("{}", err); // Se registra el error
                Err(AppError::InternalError)
            }
        }
    }
}
