#[macro_export]
macro_rules! respond_with {
    ($content_type: expr,$html: expr, $json: expr) => {
        match $content_type {
            ContentTypes::Html => Ok(JsonOrHtml::Html($html)),
            _ => Ok(JsonOrHtml::Json(Json($json))),
        }
    };
}
