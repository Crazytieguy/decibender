use std::env;

use rspotify::{clients::OAuthClient, scopes, AuthCodeSpotify, Config, Credentials, OAuth};

pub async fn init() -> anyhow::Result<AuthCodeSpotify> {
    let creds = Credentials::new(&env!("RSPOTIFY_CLIENT_ID"), &env!("RSPOTIFY_CLIENT_SECRET"));
    let oauth = OAuth {
        redirect_uri: env!("RSPOTIFY_REDIRECT_URI").to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-read-currently-playing",
            "user-modify-playback-state",
            "playlist-read-private"
        ),
        ..Default::default()
    };
    let cache_path = tauri::api::path::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get cache dir"))?
        .join(".spotify_token_cache.json");
    let config = Config {
        token_cached: true,
        cache_path,
        ..Default::default()
    };
    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&url).await?;
    Ok(spotify)
}
