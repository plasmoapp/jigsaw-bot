use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct ReportResposnse(eyre::Report);

impl IntoResponse for ReportResposnse {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ReportResposnse
where
    E: Into<eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
