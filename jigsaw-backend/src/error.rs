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

impl<E> From<E> for ReportResposnse
where
    E: Into<eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
