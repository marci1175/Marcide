use std::time::SystemTime;
mod dependencies;
use dependencies::{self as es, anyhow, ds, tokio, tracing};

#[tokio::main]
pub async fn rpc(projname : String, projstart : String) -> Result<(), anyhow::Error> {
    let client = es::make_client(ds::Subscriptions::ACTIVITY).await;

    let mut activity_events = client.wheel.activity();

    tokio::task::spawn(async move {
        while let Ok(ae) = activity_events.0.recv().await {
            tracing::info!(event = ?ae, "received activity event");
        }
    });

    let rp = ds::activity::ActivityBuilder::default()
        .details("Marcide, made by marci1175")
        .state(format!("Session started : {}", projstart).to_owned())
        .assets(
            ds::activity::Assets::default()
                .large("the".to_owned(), Some("u mage".to_owned()))
                .small("the".to_owned(), Some("i mage".to_owned())),
        )
        .button(ds::activity::Button {
            label: "Marcide Official github".to_owned(),
            url: "https://github.com/marci1175/marcide".to_owned(),
        })
        .button(ds::activity::Button {
            label: "Marcide Official discord".to_owned(),
            url: "https://discord.gg/hT9JdwgbQv".to_owned(),
        })
        .start_timestamp(SystemTime::now());

    tracing::info!(
        "updated activity: {:?}",
        client.discord.update_activity(rp).await
    );

    let mut r = String::new();
    let _ = std::io::stdin().read_line(&mut r);

    tracing::info!(
        "cleared activity: {:?}",
        client.discord.clear_activity().await
    );

    client.discord.disconnect().await;

    Ok(())
}
