use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub(crate) mod davinci;

pub(crate) enum AppError {
  InternalServerError(anyhow::Error),
  PlanUnavailable,
}

impl From<anyhow::Error> for AppError {
  fn from(inner: anyhow::Error) -> Self {
    AppError::InternalServerError(inner)
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, error_message) = match self {
      AppError::InternalServerError(inner) => {
        error!("stacktrace: {}", inner);
        (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
      }
      AppError::PlanUnavailable => (
        StatusCode::SERVICE_UNAVAILABLE,
        "substitution plan is currently unavailable",
      ),
    };

    (status, error_message).into_response()
  }
}
