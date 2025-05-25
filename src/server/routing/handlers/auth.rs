use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Json},
};
use tower_cookies::{Cookie, Cookies};
use serde::Deserialize;
use crate::utils::config::Config;

/// Form data for login requests.
#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}

/// Password-protected login route and create authentication cookie.
pub async fn login(cookies: Cookies, Form(form): Form<LoginForm>) -> impl IntoResponse {
    if form.password == Config::from_env().password().to_owned() {
        let mut cookie = Cookie::new("auth", "1");
        cookie.set_path("/");
        cookie.set_max_age(cookie::time::Duration::hours(12)); // 12 hours
        cookies.add(cookie);
        Redirect::to("/master").into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Wrong access code").into_response()
    }
}

/// Checks if the user is authenticated using the cookie created by the login route.
/// If not authenticated, redirects to the error page.
pub async fn master_protection(cookies: Cookies) -> impl IntoResponse {
    let password_required = !Config::from_env().password().is_empty();
    let has_auth_cookie = cookies.get("auth").map(|c| c.value().to_owned()) == Some("1".to_string());

    if !password_required || has_auth_cookie {
        let content = std::fs::read_to_string("static/html/master.html")
            .or_else(|_| std::fs::read_to_string("static/html/error.html"))
            .unwrap_or_else(|_| "<h1>Page not found</h1>".to_string());
        Html(content).into_response()
    } else {
        Redirect::to("static/html/error.html").into_response()
    }
}

/// Checks if a password is required for authentication.
pub async fn password_required() -> Json<bool> {
    let required = !Config::from_env().password().is_empty();
    Json(required)
}
