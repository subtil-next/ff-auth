# ff-auth
Rust tools that provide auth information for the different versions of FFXIV.

This package only contains patterns for authing into different regions. 


# Global

```rust
use ff_auth::prelude::*;

fn auth() {
    let client = GlobalClient::default();
    let response = client.authenticate(LoginRequest::new(reqwest).with_username("username").with_password("password"));
}
```

# SteamClient (Global)
```rust
use ff_auth::prelude::*;
fn auth(){
 
    let client = SteamClient::default();
    let response = client.authenticate(LoginRequest::new(reqwest));
}
```

If you are seeing errors like STATUS_DLL_NOT_FOUND, Image not found etc. You are likely missing the Steamworks SDK Redistributable files. The libraries need to exist somewhere the operating system can find them. This is likely next to your binary (.exe on windows).