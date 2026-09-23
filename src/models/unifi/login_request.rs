use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub remember_me: bool,
    pub token: &'a str,
}
