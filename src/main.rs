use user_api_server::layers::ewi::setup::setup_and_run;


#[tokio::main]
async fn main() {
    if let Err(err) =  setup_and_run() .await {
        panic!("failed to initialize server due to error: {:?}", err);
    }
}