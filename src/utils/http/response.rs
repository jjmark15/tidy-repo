use http_types::StatusCode;
use serde_json::Error;

#[derive(Debug, Clone)]
pub struct Response {
    status_code: StatusCode,
    body_string: String,
}

impl Response {
    pub fn new(status_code: StatusCode, body_string: String) -> Self {
        Response {
            status_code,
            body_string,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn body_string(&self) -> &String {
        &self.body_string
    }

    pub fn body_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str(self.body_string.as_str())
    }
}

impl Into<Response> for surf::Response {
    fn into(self) -> Response {
        async_std::task::block_on(response_from_surf_response(self))
    }
}

async fn response_from_surf_response(mut surf_response: surf::Response) -> Response {
    Response::new(
        surf_response.status(),
        surf_response.body_string().await.unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    #[test]
    fn returns_status_code() {
        let under_test = Response::new(StatusCode::Ok, "body_string".to_string());
        assert_that(&under_test.status_code()).is_equal_to(&StatusCode::Ok);
    }

    #[test]
    fn returns_body_string() {
        let under_test = Response::new(StatusCode::Ok, "body string".to_string());
        assert_that(&under_test.body_string()).is_equal_to(&"body string".to_string());
    }

    #[test]
    fn returns_body_json() {
        let under_test = Response::new(StatusCode::Ok, "1".to_string());
        assert_that(&under_test.body_json().unwrap()).is_equal_to(1);
    }
}
