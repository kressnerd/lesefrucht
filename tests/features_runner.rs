mod steps;

use cucumber::World as _;
use cucumber::cli::Opts;

#[tokio::main]
async fn main() {
    steps::support::world::World::cucumber()
        .fail_on_skipped()
        .max_concurrent_scenarios(Some(1))
        .with_cli(Opts::<_, _, _>::parsed())
        .run_and_exit("givn")
        .await;
}
