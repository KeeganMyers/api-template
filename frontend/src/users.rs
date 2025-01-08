use axum::response::Html;
use minijinja::context;
use model::{user_readmodel::UserReadModel, State};
use util::{error::UtilError, store::PaginatedResult};

pub async fn get_users(
    paginated_users: PaginatedResult<UserReadModel>,
    state: &State,
) -> Result<Html<String>, UtilError> {
    if let Some(env) = &state.template_env {
        let template = env.get_template("users.html")?;
        return Ok(Html(
            template.render(context!(users => paginated_users.data ))?,
        ));
    }
    Err(UtilError::TemplatesNotLoaded)
}

pub async fn get_user(user: UserReadModel, state: &State) -> Result<Html<String>, UtilError> {
    if let Some(env) = &state.template_env {
        let template = env.get_template("user.html")?;

        return Ok(Html(template.render(context!(user => user ))?));
    }
    Err(UtilError::TemplatesNotLoaded)
}
