use steamworks::{AppId, Client};
use tracing::instrument;
use crate::clients::global_utils::{get_oauth_login, Ticket};
use crate::prelude::{AuthProvider, LoginRequest, LoginResponse};

#[derive(Debug, Default)]
struct SteamClient{

}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct GlobalClient{

}


/*  public const uint STEAM_APP_ID = 39210;
        public const uint STEAM_FT_APP_ID = 312060;

 */

const FREE_TRIAL_APP_ID: AppId = AppId(312060);
const STEAM_APP_ID: AppId = AppId(39210);

#[async_trait::async_trait]
impl AuthProvider for GlobalClient {
    #[instrument(name="GlobalClient::authenticate", ret, err)]
    async fn authenticate(&self, req: LoginRequest) -> crate::error::Result<LoginResponse> {
        let ticket = match req.is_free_trial {
            Some(true) => Ticket::new(FREE_TRIAL_APP_ID)?,
            _ => Ticket::new(STEAM_APP_ID)?,
        };

        get_oauth_login(req, Some(ticket)).await
    }
}
