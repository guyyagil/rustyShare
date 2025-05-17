use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum::{Form, extract::Query};
use axum::response::{IntoResponse, Html};


#[derive(serde::Deserialize)]
struct LoginForm {
    password: String,
}

async fn login(Form(form): Form<LoginForm>, jar: CookieJar) -> impl IntoResponse {
    if form.password == "" {
        let mut cookie = Cookie::new("auth", "1");
        cookie.set_path("/");
        (jar.add(cookie), axum::response::Redirect::to("/media"))
    } else {
        (jar, axum::response::Redirect::to("/"))
    }
}

