use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt, join};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    token: String,
}

impl Config {
    async fn get() -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = File::open("config.json").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InviteData {
    pub r#type: i64,
    pub code: String,
    pub inviter: Option<Inviter>,
    pub expires_at: Option<String>,
    pub flags: i64,
    pub guild: Guild,
    pub guild_id: String,
    pub channel: Channel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Inviter {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub discriminator: String,
    pub public_flags: u64,
    pub flags: u64,
    pub bot: Option<bool>,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    pub global_name: Option<String>,
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    pub banner_color: Option<String>,
    pub clan: Option<Clan>,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AvatarDecorationData {
    pub asset: String,
    pub sku_id: String,
    pub expires_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Clan {
    pub identity_guild_id: String,
    pub identity_enabled: bool,
    pub tag: String,
    pub badge: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Guild {
    pub id: String,
    pub name: String,
    pub splash: Option<String>,
    pub banner: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub features: Vec<String>,
    pub verification_level: i64,
    pub vanity_url_code: Option<String>,
    pub nsfw_level: i64,
    pub nsfw: bool,
    pub premium_subscription_count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Channel {
    pub id: String,
    pub r#type: i64,
    pub name: String,
}

impl InviteData {
    async fn get(token: &str, link: &str) -> Result<InviteData, Box<dyn std::error::Error>> {
        let client = Client::new();
        let mut header = HeaderMap::new();
        header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_value = format!("Bot {}", token);
        header.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        let res = client.get(&format!("https://discord.com/api/v10/invites/{}", link))
            .headers(header)
            .send().await?;
        let body = res.text().await?;
        dbg!(&body);
        let data: InviteData = serde_json::from_str(&body)?;
        Ok(data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let config = Config::get().await?;

    loop {
        let link = rl.readline("Invite-Link: ")?.replace("https://discord.gg/", "").replace("https://discord.com/invite/", "");
        println!();


        let invite_data = InviteData::get(&config.token, &link).await?;
        println!("{:#?}", invite_data);


        println!();
    }
    Ok(())
}
