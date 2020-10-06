use structopt::StructOpt;

use tidy_repo::ports::cli::structopt::StructOptClientOptions;
use tidy_repo::TidyRepoClient;

fn main() {
    let client_options = StructOptClientOptions::from_args();
    TidyRepoClient::new(client_options);
}
