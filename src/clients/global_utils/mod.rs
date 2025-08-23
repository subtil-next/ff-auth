use std::collections::HashMap;
use std::sync::LazyLock;
use reqwest::{header, RequestBuilder};
use scraper::{Html, Selector};
use tracing::{debug, info, instrument, trace};
use url::Url;
use crate::prelude::{LoginRequest, LoginResponse};
#[cfg(feature = "steam")]
mod crt_rand;
#[cfg(feature = "steam")]
pub(crate) use crt_rand::*;

mod headers;
mod ticket;

pub(crate) use ticket::*;
pub(crate) use headers::*;
use crate::error::Error;



fn get_oauth_top_url(req: &LoginRequest, steam: Option<Ticket>) -> crate::error::Result<Url> {
    let mut params = Vec::new();

    params.push(("lng", "en".to_string()));//TODO: enable customization for the lng
    if let Some(r) = req.region {
        params.push(("rgn", r.to_string()));
    }

    params.push(("isft",  if req.is_free_trial.unwrap_or(false) {
        "1".to_string()
    } else {
        "0".to_string()
    }));

    params.push(("cssmode", "1".to_string()));
    params.push(("isnew", "1".to_string()));
    params.push(("launchver", "3".to_string()));

    if let Some(steam) = steam {
        params.push(("issteam", "1".to_string()));
        params.push(("session_ticket", steam.text));
        params.push(("ticket_size", steam.length.to_string()));
    }

    Ok(Url::parse_with_params("https://ffxiv-login.square-enix.com/oauth/ffxivarr/login/top", params)?)
}


pub(crate) async fn get_oauth_login(mut req: LoginRequest, steam: Option<Ticket>) -> crate::error::Result<LoginResponse> {
    if steam.is_none() {
        if req.username.is_none() {
            return Err(Error::MissingUsername);
        }
        if req.password.is_none() {
            return Err(Error::MissingPassword);
        }
    } else {
        // for steam requests we do not use the username or password
        req.username.take();
        req.password.take();
    }

    let url = get_oauth_top_url(&req, steam)?;

    let (action, method, mut input) = get_oauth_top(req.client.clone(), url.clone()).await?;
    if method != "post".to_string() ||  input.is_empty() {
        return Err(Error::MissingLoginForm);
    }

    let login_url = url.join(&action)?;

    info!("Performing OAuth login");
    let builder = req.client.post(login_url)
        .default_ffxiv_headers()
        .header(header::REFERER, url.as_str())
        .header(header::COOKIE, "_rsid=\"\"");

    if let Some(username) = req.username {
        input.insert("sqexid".to_string(), username);
    }
    if let Some(password) = req.password {
        input.insert("password".to_string(), password.into_unsecure());
    }
    if let Some(otp) = req.otp {
        input.insert("otppw".to_string(), otp);
    }
    
    let res = builder.form(&input).send().await?.text().await?;

    let Some(params) = extract_launch_params(&res) else {
        return Err(Error::LoginFailure);
    };
    if params["auth"] != "ok" {
        return Err(Error::LoginFailureMessage(params["err"].to_string()));
    }

    Ok(LoginResponse{
        session_id: params["sid"].to_string(),
        region: params["region"].parse()?,
        terms_accepted: params["terms"] != "0",
        playable: params["playable"] != "0",
        max_expansion: params["maxex"].parse()?,
    })
}


async fn get_oauth_top(client: reqwest::Client, url: Url) -> crate::error::Result<(String, String, HashMap<String, String>)> {

    let builder = client.get(url)
        .default_ffxiv_headers()
        .header(header::COOKIE, "_rsid=\"\"");


    let text = builder.send().await?.text().await?;

    if text.contains("window.external.user(\"restartup\");") {
        return Err(Error::Restartup)
    }
    let document = Html::parse_document(&text);
    let form_selector = Selector::parse("form[name=mainForm]").unwrap();
    let input_selector = Selector::parse("input").unwrap();


    //extract form elements from "mainForm"
    if let Some(form) = document.select(&form_selector).next() {
        let action = form.value().attr("action").unwrap_or_default().to_string();
        let method = form.value().attr("method").unwrap_or_default().to_string();
        let mut form_data = HashMap::new();

        for input in form.select(&input_selector) {
            if let Some(name) = input.value().attr("name") {
                if let Some(value) = input.value().attr("value") {
                    form_data.insert(name.to_string(), value.to_string());
                }
            }
        }
        Ok((action, method, form_data))
    } else {
        Err(Error::MissingLoginForm)
    }

}


fn extract_launch_params(html: &str) -> Option<HashMap<String, String>> {
    const FULL_START: &str = r#"window.external.user(""#;
    const END: &str = r#"");"#;

    html.match_indices(FULL_START)
        .find(|(start_idx, _)| !is_in_comment(html, *start_idx))
        .and_then(|(start_idx, _)| {
            let params_start = start_idx + FULL_START.len();
            html[params_start..]
                .find(END)
                .map(|end| html[params_start..params_start + end].to_string())
        })
        .and_then(|params_str| {
            // Check if it starts with "login=" and parse accordingly
            info!("Extracted launch params {:?}", params_str);
            if params_str.starts_with("login=") {
                let params_str = &params_str["login=".len()..];
                parse_login_params(params_str)
            } else {
                None
            }
        })
}

/// parse_login_params converts a login=key,value,key,value,key,value to a dictionary.
fn parse_login_params(params_str: &str) -> Option<HashMap<String, String>> {
    let parts: Vec<&str> = params_str.split(',').collect();

    // Need at least "login=XXX" and one more part
    if parts.len() < 2 {
        return None;
    }

    let mut result = HashMap::new();

    for chunk in parts.chunks(2) {
        if chunk.len() == 2 {
            let key = chunk[0].to_string();
            let value = chunk[1].to_string();
            result.insert(key, value);
        }
    }

    Some(result)
}

#[instrument(skip(html), ret)]
fn is_in_comment(html: &str, position: usize) -> bool {
    let before_position = &html[..position];

    // Check for single-line comment
    if let Some(line_start) = before_position.rfind('\n') {
        let line_prefix = &html[line_start + 1..position];
        if line_prefix.trim_start().starts_with("//") {
            return true;
        }
    } else {
        // First line case
        if before_position.trim_start().starts_with("//") {
            return true;
        }
    }
    // false
    // Check for multi-line comment
    let last_comment_start = before_position.rfind("/*");
    let last_comment_end = before_position.rfind("*/");

    match (last_comment_start, last_comment_end) {
        (Some(start), Some(end)) => start > end, // Inside comment if last /* is after last */
        (Some(_), None) => true,                 // Inside comment if /* found but no closing */
        _ => false,                              // Not in comment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_launch_params() {
        let content = include_str!("test_content.html");

        let res = extract_launch_params(content);
        assert!(res.is_some());
    }
}