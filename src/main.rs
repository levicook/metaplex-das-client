#[cfg(feature = "cli")]
fn main() {
    cli::async_main();
}

#[cfg(feature = "cli")]
mod cli {
    use std::sync::Arc;

    use clap::{command, Arg, ArgMatches, Command};
    use metaplex_das_client::{DasClient, RateLimiter};

    #[tokio::main]
    pub(crate) async fn async_main() {
        let matches = command!()
            .subcommand_required(true)
            .arg_required_else_help(true)
            .arg(Arg::new("url").short('u').long("url").required(true))
            .subcommand(
                Command::new("get-asset") //
                    .arg(Arg::new("asset_id").long("asset")),
            )
            .subcommand(
                Command::new("get-assets-by-owner") //
                    .arg(Arg::new("owner_address").long("owner")),
            )
            .get_matches();

        match matches.subcommand() {
            Some(("get-asset", sub_args)) => get_asset(&matches, sub_args).await,
            Some(("get-assets-by-owner", sub_args)) => {
                get_assets_by_owner(&matches, sub_args).await
            }
            _ => unreachable!(),
        }
    }

    async fn get_asset(cmd_args: &ArgMatches, sub_args: &ArgMatches) {
        let url = cmd_args.get_one::<String>("url").unwrap().to_string();
        let das_client = DasClient::new(url, new_http_client(), new_rate_limiter());

        let asset_id = sub_args.get_one::<String>("asset").unwrap().to_string();
        das_client.get_asset(&asset_id).await.unwrap();
    }

    async fn get_assets_by_owner(cmd_args: &ArgMatches, sub_args: &ArgMatches) {
        let url = cmd_args.get_one::<String>("url").unwrap().to_string();
        let das_client = DasClient::new(url, new_http_client(), new_rate_limiter());

        let owner_address = sub_args
            .get_one::<String>("owner_address")
            .unwrap()
            .to_string();
        das_client
            .get_assets_by_owner(&owner_address, 1, 1000)
            .await
            .unwrap();
    }

    fn new_http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    fn new_rate_limiter() -> RateLimiter {
        Arc::new(governor::RateLimiter::direct(new_quota()))
    }

    fn new_quota() -> governor::Quota {
        governor::Quota::per_second(std::num::NonZeroU32::new(100u32).unwrap())
    }
}
