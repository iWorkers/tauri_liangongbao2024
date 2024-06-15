use reqwest::{Client, Response, StatusCode};
use std::collections::HashMap;
use rand::Rng;
use crate::random_user_agent::set_user_agent;

pub struct HTTPClient {
    client: Client,
    token: String,
    member_id: String,
    cookie: HashMap<String, String>,
    user_agent: String,
}

impl HTTPClient {
    pub fn new() -> Self {
        let client = Client::new();
        let user_agent = set_user_agent();

        HTTPClient {
            client,
            token: String::new(),
            member_id: String::new(),
            cookie: HashMap::new(),
            user_agent,
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = token.to_string();
    }

    pub fn set_member_id(&mut self, member_id: &str) {
        self.member_id = member_id.to_string();
    }

    pub fn set_cookie(&mut self, cookie: HashMap<String, String>) {
        self.cookie = cookie;
    }

    pub fn send(&self, urls: &HashMap<&str, &str>, data: Option<&str>) -> Result<Response, reqwest::Error> {
        let req_url = urls.get("req_url").unwrap_or(&"");
        let method = urls.get("req_type").unwrap_or(&"GET");
        let is_json = urls.get("is_json").unwrap_or(&"false") == "true";

        let mut request_builder = self.client.request(
            method.parse().unwrap(),
            req_url,
        );

        if let Some(data) = data {
            request_builder = request_builder.body(data.to_string());
        }

        request_builder = request_builder
            .header("User-Agent", &self.user_agent)
            .header("token", &self.token)
            .header("memberId", &self.member_id);

        for (key, value) in &self.cookie {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder.send()?;

        if response.status() == StatusCode::OK {
            if is_json {
                let json_response: serde_json::Value = response.json()?;
                Ok(response)
            } else {
                let text_response = response.text()?;
                Ok(response)
            }
        } else {
            Err(reqwest::Error::from(response))
        }
    }
}
