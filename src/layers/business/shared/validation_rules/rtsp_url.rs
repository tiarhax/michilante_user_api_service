use url::Url;

use crate::layers::business::shared::business_rules::FieldValidationResult;

pub fn rtsp_url(input: &str, field_name: &str, message: &str) -> FieldValidationResult {
    match is_valid_rtsp_url(input) {
        true => FieldValidationResult::Valid,
        false => FieldValidationResult::Invalid(field_name.to_string(), message.to_string()),
    }
}

fn is_valid_rtsp_url(input: &str) -> bool {
    match Url::parse(input) {
        Ok(url) => url.scheme() == "rtsp" || url.scheme() == "rtsps",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rtsp_urls() {
        assert!(is_valid_rtsp_url("rtsp://example.com/live/stream"));
        assert!(is_valid_rtsp_url("rtsps://secure.example.com/live/stream"));
        assert!(is_valid_rtsp_url("rtsp://192.168.1.1:1935/app/stream"));
        assert!(is_valid_rtsp_url("rtsp://user:password@192.168.1.1:1935/app/stream"));
    }

    #[test]
    fn test_invalid_rtsp_urls() {
        assert!(!is_valid_rtsp_url("http://example.com"));
        assert!(!is_valid_rtsp_url("https://example.com"));
        assert!(!is_valid_rtsp_url("ftp://example.com"));
        assert!(!is_valid_rtsp_url("not_a_url"));
        assert!(!is_valid_rtsp_url(""));
    }
}