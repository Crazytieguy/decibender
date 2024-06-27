use rspotify::{clients::OAuthClient, scopes, AuthCodeSpotify, Config, Credentials, OAuth};

pub async fn init() -> anyhow::Result<AuthCodeSpotify> {
    let creds = Credentials::from_env()
        .ok_or_else(|| anyhow::anyhow!("Failed building spotify credentials from env"))?;
    let oauth = OAuth::from_env(scopes!(
        "user-read-playback-state",
        "user-read-currently-playing",
        "user-modify-playback-state",
        "playlist-read-private"
    ))
    .ok_or_else(|| anyhow::anyhow!("Failed building spotify oauth from env"))?;
    let spotify = AuthCodeSpotify::with_config(
        creds,
        oauth,
        Config {
            token_cached: true,
            ..Default::default()
        },
    );
    let url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&url).await?;
    Ok(spotify)
}
