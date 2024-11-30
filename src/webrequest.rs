use clap::{Parser, ValueEnum};
use reqwest::{Client, Error, Method, Request, Response, Url};

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "url", required = true)]
    r#url: String,

    #[arg(short = 'd', long = "data", help = "HTTP Post data")]
    r#data: Option<String>,

    #[arg(short = 'A', long = "user-agent", help = "Send User-Agent <name> to server")]
    r#user_agent: Option<String>,

    #[arg(short = 'X', long = "request", value_enum, default_value_t = HttpMethod::Get, help = "Specify request method to use")]
    r#method: HttpMethod,

    #[arg(short = 'H', long = "header", help = "Pass custom header(s) to server")]
    r#header: Vec<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum HttpMethod {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
}

#[derive(Debug)]
pub struct WebClient {
    client: Client,
    request: Request,
}

impl WebClient {
    pub fn build(args: &[String]) -> anyhow::Result<Self> {
        let arg = Args::try_parse_from(args)?;

        let url = {
            let url_str = {
                if arg.url.contains("://") {
                    arg.url
                } else {
                    format!("http://{}", arg.url)
                }
            };
            Url::parse(&url_str)?
        };

        let client = {
            let mut c = Client::builder();

            if let Some(useragent) = arg.user_agent { c = c.user_agent(useragent) }

            c.build()?
        };

        let request = {
            let method = match arg.method {
                HttpMethod::Options => Method::OPTIONS,
                HttpMethod::Get => Method::GET,
                HttpMethod::Post => Method::POST,
                HttpMethod::Put => Method::PUT,
                HttpMethod::Delete => Method::DELETE,
                HttpMethod::Head => Method::HEAD,
                HttpMethod::Trace => Method::TRACE,
                HttpMethod::Connect => Method::CONNECT,
                HttpMethod::Patch => Method::PATCH,
            };
            let mut r = client.request(method, url);

            if let Some(data) = arg.data { r = r.body(data) }

            for row in arg.header {
                let h = row
                  .split(r#":"#)
                  .map(|x| x.trim())
                  .collect::<Vec<&str>>();
                r = r.header(h[0], h[1]);
            }

            r.build()?
        };

        Ok(Self {
            client,
            request,
        })
    }

    pub async fn send(&self) -> Result<Response, Error> {
        let r = self.request.try_clone().unwrap();
        self.client.execute(r).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }

    #[test]
    fn test_approve_cmd() {
        let args = Args::try_parse_from(
            [
                "cmd",
                "localhost",
                "--data", "string-data",
                "-A", "custom/user-agent",
                "-H", "Content-Type: application/json",
                "-H", "Cookie: 123456789",
            ]
        );

        assert!(args.is_ok());
        if let Ok(a) = args { println!("{:#?}", a) }
    }

    #[test]
    fn test_build() {
        let arg = vec![
                "cmd",
                "localhost",
                "--data", "string-data",
                "-A", "custom/user-agent",
                "-H", "Content-Type: application/json",
                "-H", "Cookie: 123456789",
            ].into_iter()
          .map(String::from)
          .collect::<Vec<String>>();

        let _client = WebClient::build(&arg);
    }
}
