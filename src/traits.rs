use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct LoginRequest {
    pub client: reqwest::Client,

    pub username: Option<String>,
    pub password: Option<secure_string::SecureString>,
    pub otp: Option<String>,
    pub region: Option<i8>,
    pub is_free_trial: Option<bool>,
}

impl LoginRequest {
    pub fn new(client: reqwest::Client) -> Self {
        LoginRequest {
            client,
            username: None,
            password: None,
            otp: None,
            region: None,
            is_free_trial: None,
        }
    }
    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }
    pub fn with_password<U>(mut self, password: U) -> Self
    where U: Into<String>{
        self.password = Some(secure_string::SecureString::from(password));
        self
    }
    pub fn with_otp(mut self, otp: String) -> Self {
        self.otp = Some(otp);
        self
    }
    pub fn with_region(mut self, region: i8) -> Self {
        self.region = Some(region);
        self
    }
    pub fn with_free_trial(mut self) -> Self {
        self.is_free_trial = Some(true);
        self
    }
}


impl Debug for LoginRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("client", &"[Client]")
            .field("username", &self.username.as_ref().map(|u| {
                if u.len() <= 3 {
                    u.clone()
                } else {
                    format!("{}***", u.chars().take(3).collect::<String>())
                }
            }))
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("otp", &self.otp.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct LoginResponse {
    pub(crate) session_id: String,
    pub(crate) region: u8,
    pub(crate) terms_accepted: bool,
    pub(crate) playable: bool,
    pub(crate) max_expansion: u8,
}

#[async_trait::async_trait]
pub trait AuthProvider {
    async fn authenticate(&self, req: LoginRequest) -> crate::error::Result<LoginResponse>;
}