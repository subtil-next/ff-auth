use crate::error::Error;
use crate::prelude::{LoginRequest, LoginResponse};
use crate::traits::AuthProvider;
use reqwest::{header, RequestBuilder};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{info, instrument};
use url::Url;
use crate::clients::global_utils::{get_oauth_login, DefaultHeaders};

#[derive(Debug, Default)]
pub struct GlobalClient{

}


#[async_trait::async_trait]
impl AuthProvider for GlobalClient {
    #[instrument(name="GlobalClient::authenticate", skip(req), ret, err)]
    async fn authenticate(&self, req: LoginRequest) -> crate::error::Result<LoginResponse> {
        get_oauth_login(req, None).await
    }
}