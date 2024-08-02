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
        let data: InviteData = serde_json::from_str(&body)?;
        Ok(data)
    }
    async fn get_invite_type(&self) -> Result<String, Box<dyn std::error::Error>> {
        let invite_type = match self.r#type {
            0 => "Guild Invite",
            1 => "Group DM Invite",
            2 => "Friend Invite",
            _ => "Unknown Invite",
        };
        Ok(invite_type.to_string())
    }
    async fn inviter_id_to_link(&self, img_type: ImageType) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if self.inviter.is_none() {
            return Ok(None);
        }
        let inviter = self.inviter.clone().unwrap();
        let img_id;
        if img_type == ImageType::Avatar && inviter.avatar.is_none() {
            return Ok(Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()));
        } else if img_type == ImageType::Banner && inviter.banner.is_none() {
            return Ok(Some("None".to_string()));
        } else {
            img_id = match img_type {
                ImageType::Avatar => inviter.avatar.clone().unwrap(),
                ImageType::Banner => inviter.banner.clone().unwrap(),
                ImageType::AvatarDecoration => inviter.avatar_decoration_data.clone().unwrap().asset,
            };
        }
        let mut url = String::new();
        if img_type == ImageType::Avatar || img_type == ImageType::Banner {
            url = format!("https://cdn.discordapp.com/{}/{}/{}", &img_type, inviter.id, img_id)
        } else if img_type == ImageType::AvatarDecoration {
            return Ok(Some(format!("https://cdn.discordapp.com/{}/{}.png?size=4096", &img_type, img_id)));
        }

        url.push_str(".gif");
        let response = reqwest::get(&url).await?;
        return if response.status().is_success() {
            url.push_str("?size=4096");
            Ok(Some(url))
        } else {
            url.truncate(url.len() - 4);
            url.push_str(".png?size=4096");
            Ok(Some(url))
        };
    }
    async fn check_flags(&self) -> Option<Vec<String>> {
        if self.inviter.is_none() {
            return None;
        }
        let inviter = self.inviter.clone().unwrap();
        if inviter.public_flags == 0 {
            return None;
        }
        const FLAGS: &[(&str, u64)] = &[
            ("Staff", 1),
            ("Partnered Server Owner", 2),
            ("HypeSquad Events", 4),
            ("Bug Hunter Level 1", 8),
            ("HypeSquad Bravery", 64),
            ("HypeSquad Brilliance", 128),
            ("HypeSquad Balance", 256),
            ("Premium Early Supporter", 512),
            ("Team Pseudo User", 1024),
            ("Bug Hunter Level 2", 16384),
            ("Verified Bot", 65536),
            ("Verified Developer", 131072),
            ("Certified Moderator", 262144),
            ("Bot Http Interactions", 524288),
            ("Active Developer", 4194304)
        ];

        let badge = FLAGS.iter()
            .filter_map(|&(flag_name, flag_value)| {
                if inviter.public_flags & flag_value == flag_value {
                    Some(flag_name.to_string())
                } else {
                    None
                }
            })
            .collect();
        return Some(badge);
    }
}

#[derive(Eq, PartialEq)]
enum ImageType {
    Avatar,
    Banner,
    AvatarDecoration,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageType::Avatar => write!(f, "avatars"),
            ImageType::Banner => write!(f, "banners"),
            ImageType::AvatarDecoration => write!(f, "avatar-decoration-presets"),
        }
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


        let (invite_type, avatar, banner, asset, badge) = join!(
            invite_data.get_invite_type(),
            invite_data.inviter_id_to_link(ImageType::Avatar),
            invite_data.inviter_id_to_link(ImageType::Banner),
            invite_data.inviter_id_to_link(ImageType::AvatarDecoration),
            invite_data.check_flags()
        );


        println!("Invite:");
        println!(" - Type: {}", invite_type?);
        println!(" - Code: {}", link);
        println!(" - Expires at: {}", invite_data.expires_at.unwrap_or("Life Time".to_string()));
        println!(" - Flags: {}", invite_data.flags);
        println!(" - Guild ID: {}", invite_data.guild_id);

        if invite_data.inviter.is_some() {
            let inviter_data = invite_data.inviter.clone().unwrap();
            println!("Inviter:");
            println!(" - ID: {}", inviter_data.id);
            println!(" - Username: {}", inviter_data.username);
            println!(" - Avatar: {}", avatar?.unwrap_or("None".to_string()));
            println!(" - Discriminator: {}", inviter_data.discriminator);
            println!(" - Public Flags: {}", inviter_data.public_flags);
            if badge.is_some() {
                println!(" - Badge:");
                for flag in badge.unwrap() {
                    println!(" -  - {}", flag);
                }
            } else {
                println!(" - Badge: None");
            }
            println!(" - Flags: {}", inviter_data.flags);
            println!(" - Banner: {}", banner?.unwrap_or("None".to_string()));
            println!(" - Bot: {}", inviter_data.bot.unwrap_or(false));
            println!(" - Banner: {}", inviter_data.banner.unwrap_or("None".to_string()));
            if inviter_data.accent_color.is_some() {
                println!(" - Accent Color: {}", format!("#{:06x}", inviter_data.accent_color.unwrap()));
            } else {
                println!(" - Accent Color: None");
            }
            println!(" - Global Name: {}", inviter_data.global_name.unwrap_or("None".to_string()));
            if inviter_data.avatar_decoration_data.is_some() {
                let avatar_decoration_data = inviter_data.avatar_decoration_data.clone().unwrap();
                println!(" - Avatar Decoration Data:");
                println!(" -  - Asset: {}", asset?.unwrap_or("None".to_string()));
                println!(" -  - SKU ID: {}", avatar_decoration_data.sku_id);
                if avatar_decoration_data.expires_at.is_some() {
                    println!(" -  - Expires at: {}", avatar_decoration_data.expires_at.unwrap());
                } else {
                    println!(" -  - Expires at: None");
                }
            } else {
                println!(" - Avatar Decoration Data: None");
            }
            println!(" - Banner Color: {}", inviter_data.banner_color.unwrap_or("None".to_string()));
            if inviter_data.clan.is_some() {
                let clan = inviter_data.clan.clone().unwrap();
                println!(" - Clan:");
                println!(" -  - Identity Guild Id: {}", clan.identity_guild_id);
                println!(" -  - Identity Enabled: {}", clan.identity_enabled);
                println!(" -  - Tag: {}", clan.tag);
                println!(" -  - Badge: {}", clan.badge);
            } else {
                println!(" - Clan: None");
            }
        } else {
            println!("Inviter: None");
        }

        println!("Guild:");
        println!(" - ID: {}", invite_data.guild.id);
        println!(" - Name: {}", invite_data.guild.name);
        println!(" - Splash: {}", invite_data.guild.splash.unwrap_or("None".to_string()));
        println!(" - Banner: {}", invite_data.guild.banner.unwrap_or("None".to_string()));
        println!(" - Description: {}", invite_data.guild.description.unwrap_or("None".to_string()));
        println!(" - Icon: {}", invite_data.guild.icon.unwrap_or("None".to_string()));
        if !invite_data.guild.features.is_empty() {
            println!(" - Features:");
            for feature in &invite_data.guild.features {
                println!(" -  - {}", feature);
            }
        } else {
            println!(" - Features: None");
        }
        println!(" - Verification Level: {}", invite_data.guild.verification_level);
        println!(" - Vanity URL Code: {}", invite_data.guild.vanity_url_code.unwrap_or("None".to_string()));
        println!(" - NSFW Level: {}", invite_data.guild.nsfw_level);
        println!(" - NSFW: {}", invite_data.guild.nsfw);
        println!(" - Premium Subscription Count: {}", invite_data.guild.premium_subscription_count);

        println!("Channel:");
        println!(" - ID: {}", invite_data.channel.id);
        println!(" - Type: {}", invite_data.channel.r#type);
        println!(" - Name: {}", invite_data.channel.name);

        println!();
    }
    Ok(())
}
