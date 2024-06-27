use std::{env, sync::OnceLock};

struct Lights {
    on_url: String,
    off_url: String,
}

static LIGHTS: OnceLock<Lights> = OnceLock::new();

pub async fn on() -> anyhow::Result<()> {
    let lights = LIGHTS.get_or_try_init(|| {
        Ok::<_, anyhow::Error>(Lights {
            on_url: env::var("LIGHTS_ON_URL")?,
            off_url: env::var("LIGHTS_OFF_URL")?,
        })
    })?;
    reqwest::get(&lights.on_url).await?;
    Ok(())
}

pub async fn off() -> anyhow::Result<()> {
    let lights = LIGHTS.get_or_try_init(|| {
        Ok::<_, anyhow::Error>(Lights {
            on_url: env::var("LIGHTS_ON_URL")?,
            off_url: env::var("LIGHTS_OFF_URL")?,
        })
    })?;
    reqwest::get(&lights.off_url).await?;
    Ok(())
}
