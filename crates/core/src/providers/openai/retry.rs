use reqwest::StatusCode;

use crate::error::CoreError;

pub(super) fn should_retry(status: StatusCode) -> bool {
    matches!(status, StatusCode::TOO_MANY_REQUESTS)
        || status.is_server_error()
        || status == StatusCode::REQUEST_TIMEOUT
}

pub(super) fn is_unsupported_param(err: &CoreError, param: &str) -> bool {
    let message = err.to_string().to_lowercase();
    let param = param.to_lowercase();
    (message.contains("unsupported_parameter") || message.contains("unsupported parameter"))
        && message.contains(&param)
}
