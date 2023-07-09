extern crate reqwest;

use reqwest::header;
use serde::Serialize;
use serde_json::Value;

use crate::types::{FetchedUser, RawGuildUser, RawUser};

/// Checks if the user is an admin of the guild.
/// Returns false if the user is not an admin or if the request fails.
/// Otherwise returns true.
pub async fn admin_check(discord_token: &str, guild_id: &str) -> bool {
    let admin_permissions = 2147483647;

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        match header::HeaderValue::from_str(&("Bearer ".to_owned() + discord_token)) {
            Ok(s) => s,
            Err(_) => return false,
        },
    );

    let client = match reqwest::Client::builder().default_headers(headers).build() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let res = client
        .get("https://discord.com/api/users/@me/guilds")
        .send()
        .await;
    #[allow(unused_assignments)]
    let mut body = String::new();

    match res {
        Ok(res) => {
            body = res.text().await.unwrap_or("".to_string());

            let json: Value = match serde_json::from_str(&body) {
                Ok(s) => s,
                Err(_) => return false,
            };

            for guild in json.as_array().unwrap_or(&vec![]) {
                let current_guild_id = guild["id"].as_str().unwrap_or("0");
                let permissions = &guild["permissions"];

                if guild_id == current_guild_id && permissions == admin_permissions {
                    return true;
                }
            }
        }
        Err(_) => {
            return false;
        }
    }

    false
}

/// Gets all the users in a guild.
/// Returns an empty vector if the request fails.
/// If a user cannot be parsed, it will insert an "empty" user.
/// Otherwise returns a vector of users.
pub async fn get_users(discord_token: &str, guild_id: &str) -> Vec<RawGuildUser> {
    let mut users: Vec<RawGuildUser> = vec![];

    let mut after: String = "0".to_string();
    let mut keep_going = true;

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        match header::HeaderValue::from_str(&("Bot ".to_owned() + discord_token)) {
            Ok(s) => s,
            Err(_) => return users,
        },
    );

    let client = match reqwest::Client::builder().default_headers(headers).build() {
        Ok(s) => s,
        Err(_) => return users,
    };

    while keep_going {
        let res = client
            .get(
                &("https://discord.com/api/guilds/".to_owned()
                    + guild_id
                    + "/members?limit=1000&after="
                    + after.as_ref()),
            )
            .send()
            .await;
        #[allow(unused_assignments)]
        let mut body = String::new();

        match res {
            Ok(res) => {
                body = res.text().await.unwrap_or("".to_string());

                let json: Value = match serde_json::from_str(&body) {
                    Ok(s) => s,
                    Err(_) => return users,
                };

                let last = json.as_array().unwrap_or(&vec![Value::Null]).len() - 1;

                after = json.as_array().unwrap_or(&vec![])[last]["user"]["id"]
                    .as_str()
                    .unwrap_or("0")
                    .to_string();

                for user in json.as_array().unwrap_or(&vec![]) {
                    let user: RawGuildUser = match serde_json::from_value(user.clone()) {
                        Ok(s) => s,
                        Err(_) => RawGuildUser {
                            avatar: Some("".to_string()),
                            communication_disabled_until: Some("".to_string()),
                            deaf: false,
                            flags: 0,
                            joined_at: "".to_string(),
                            mute: false,
                            nick: Some("".to_string()),
                            pending: false,
                            premium_since: Some("".to_string()),
                            roles: vec![],
                            user: RawUser {
                                accent_color: Some(0),
                                avatar: Some("".to_string()),
                                avatar_decoration: Some("".to_string()),
                                banner: Some("".to_string()),
                                banner_color: Some(0),
                                bot: Some(false),
                                discriminator: "".to_string(),
                                display_name: Some("".to_string()),
                                flags: 0,
                                global_name: Some("".to_string()),
                                id: "".to_string(),
                                public_flags: 0,
                                username: "".to_string(),
                            },
                        },
                    };
                    users.push(user);
                }

                if json.as_array().unwrap_or(&vec![]).len() < 1000 {
                    keep_going = false;
                }
            }
            Err(_) => {
                keep_going = false;
            }
        }
    }

    users
}

/// Fetches a single user from the Discord API.
/// Returns None if the request fails, or the user cannot be parsed.
/// Otherwise returns the user.
pub async fn fetch_single_user(discord_token: &str, user_id: &str) -> Option<FetchedUser> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        match header::HeaderValue::from_str(&("Bot ".to_owned() + discord_token)) {
            Ok(s) => s,
            Err(_) => return None,
        },
    );

    let client = match reqwest::Client::builder().default_headers(headers).build() {
        Ok(s) => s,
        Err(_) => return None,
    };

    let res = client
        .get(&("https://discord.com/api/users/".to_owned() + user_id))
        .send()
        .await;

    #[allow(unused_assignments)]
    let mut body = String::new();

    match res {
        Ok(res) => {
            body = res.text().await.unwrap_or("".to_string());

            let json: Value = match serde_json::from_str(&body) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let user: FetchedUser = match serde_json::from_value(json) {
                Ok(s) => s,
                Err(_) => return None,
            };

            Some(user)
        }

        Err(_) => None,
    }
}

/// Gets the JSON string of a serializable object.
/// Returns an empty array string if the object cannot be serialized.
pub fn get_json_string(return_type: impl Sized + Serialize) -> String {
    match serde_json::to_string(&return_type) {
        Ok(s) => s,
        Err(e) => {
            println!("Error: {}", e);
            String::from("[]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_string() {
        let test_struct = RawGuildUser {
            avatar: Some("".to_string()),
            communication_disabled_until: Some("".to_string()),
            deaf: false,
            flags: 0,
            joined_at: "".to_string(),
            mute: false,
            nick: Some("".to_string()),
            pending: false,
            premium_since: Some("".to_string()),
            roles: vec![],
            user: RawUser {
                accent_color: Some(0),
                avatar: Some("".to_string()),
                avatar_decoration: Some("".to_string()),
                banner: Some("".to_string()),
                banner_color: Some(0),
                bot: Some(false),
                discriminator: "".to_string(),
                display_name: Some("".to_string()),
                flags: 0,
                global_name: Some("".to_string()),
                id: "".to_string(),
                public_flags: 0,
                username: "".to_string(),
            },
        };

        let json_string = get_json_string(test_struct);

        assert_eq!(
            json_string,
            "{\"avatar\":\"\",\"communication_disabled_until\":\"\",\"deaf\":false,\"flags\":0,\"joined_at\":\"\",\"mute\":false,\"nick\":\"\",\"pending\":false,\"premium_since\":\"\",\"roles\":[],\"user\":{\"accent_color\":0,\"avatar\":\"\",\"avatar_decoration\":\"\",\"banner\":\"\",\"banner_color\":0,\"bot\":false,\"discriminator\":\"\",\"display_name\":\"\",\"flags\":0,\"global_name\":\"\",\"id\":\"\",\"public_flags\":0,\"username\":\"\"}}"
        );
    }
}
