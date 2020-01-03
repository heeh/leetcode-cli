use self::req::{Json, Mode, Req};
use crate::{
    cfg::{self, Config},
    err::Error,
    plugins::chrome,
};

use std::{
    collections::HashMap,
    str::FromStr,
    time::Duration,
};

use reqwest::{
    Client,
    ClientBuilder,
    Response,
    header::{
        HeaderMap,
        HeaderName,
        HeaderValue,
    }
};

/// Leet API set
#[derive(Clone)]
pub struct LeetCode {
    pub conf: Config,
    client: Client,
    default_headers: HeaderMap,
}

impl LeetCode {
    /// Parse reqwest headers
    fn headers(mut headers: HeaderMap, ts: Vec<(&str, &str)>) -> HeaderMap {
        for (k, v) in ts.into_iter() {
            headers.insert(
                HeaderName::from_str(k).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }

        headers
    }

    /// New LeetCode client
    pub fn new() -> LeetCode {
        debug!("Building reqwest client...");
        let conf = cfg::locate();
        let cookies = chrome::cookies();
        let default_headers = LeetCode::headers(
            HeaderMap::new(),
            vec![
                ("Cookie", cookies.to_string().as_str()),
                ("x-csrftoken", &cookies.csrf),
                ("x-requested-with", "XMLHttpRequest"),
                ("Origin", &conf.sys.urls["base"])
            ],
        );
        
        let client = ClientBuilder::new()
            .gzip(true)
            .connect_timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Reqwest client build failed");

        LeetCode {
            conf,
            client,
            default_headers,
        }
    }

    /// Get category problems
    pub fn get_category_problems(self, category: &str) -> Result<Response, Error> {
        let pre_url = &self.conf.sys.urls["problems"];
        let url = &pre_url.replace("$category", category);

        Req {
            default_headers: self.default_headers,
            refer: None,
            info: false,
            json: None,
            mode: Mode::Get,
            name: "get_category_problems",
            url: url.to_string(),
        }.send(&self.client)
    }

    /// Get specific problem detail
    pub fn get_question_detail(self, slug: &str) -> Result<Response, Error> {
        let pre_refer = &self.conf.sys.urls["problems"];
        let refer = pre_refer.replace("$slug", slug);

        let mut json: Json = HashMap::new();
        json.insert(
            "query",
            vec![
                "query getQuestionDetail($titleSlug: String!) {",
                "  question(titleSlug: $titleSlug) {",
                "    content",
                "    stats",
                "    codeDefinition",
                "    sampleTestCase",
                "    enableRunCode",
                "    metaData",
                "    translatedContent",
                "  }",
                "}"
            ].join("\n")
        );

        json.insert(
            "variables",
            r#"{"titleSlug": "$titleSlug"}"#.replace("$titleSlug", &slug)
        );

        json.insert("operationName", "getQuestionDetail".to_string());
        
        Req {
            default_headers: self.default_headers,
            refer: Some(refer),
            info: false,
            json: Some(json),
            mode: Mode::Post,
            name: "get_problem_detail",
            url: (&self.conf.sys.urls["graphql"]).to_string(),
        }.send(&self.client)
    }
}


/// Sub-module for leetcode, simplify requests
mod req {
    use super::LeetCode;
    use crate::err::Error;
    use std::collections::HashMap;
    use reqwest::{
        Client,
        header::HeaderMap,
        Response,
    };

    /// Standardize json format
    pub type Json = HashMap<&'static str, String>;

    /// Standardize request mode
    pub enum Mode {
        Get,
        Post
    }

    /// LeetCode request prototype
    pub struct Req {
        pub default_headers: HeaderMap,
        pub refer: Option<String>,
        pub json: Option<Json>,
        pub info: bool,
        pub mode: Mode,
        pub name: &'static str,
        pub url: String,
    }

    impl Req {
        pub fn send<'req>(self, client: &'req Client) -> Result<Response, Error> {
            debug!("Running leetcode::{}...", &self.name);
            if self.info {
                info!("Downloading {} deps...", &self.name);
            }
            
            let headers = LeetCode::headers(
                self.default_headers,
                vec![("Referer", &self.refer.unwrap_or(self.url.to_owned()))],
            );

            let req = match self.mode {
                Mode::Get => client.get(&self.url),
                Mode::Post => client.post(&self.url).json(&self.json),
            };
            
            Ok(req.headers(headers).send()?)
        }
    }
}