use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(value_name = "url", required = true)]
    r#url: String,

    #[arg(short = 'd', long = "data", help = "HTTP Post data")]
    r#data: Option<String>,

    #[arg(short = 'A', long = "user-agent", help = "Send User-Agent <name> to server")]
    r#name: Option<String>,

    #[arg(short = 'H', long = "header", help = "Pass custom header(s) to server")]
    r#header: Vec<String>,
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
}
