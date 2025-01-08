use axum::debug_handler;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use model::State as ModelState;
use std::sync::Arc;
use util::error::UtilError;

#[debug_handler]
pub async fn index(State(state): State<Arc<ModelState>>) -> Result<Html<String>, UtilError> {
    if let Some(env) = &state.template_env {
        let template = env.get_template("index.html")?;
        return Ok(Html(template.render(context!())?));
    }
    Err(UtilError::TemplatesNotLoaded)
}
