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
        .details(format!("Current file : {}", projname))
        .state(format!("Session started : {}", projstart).to_owned())
        .assets(
            ds::activity::Assets::default()
                .large("dsicordmariced".to_owned(), Some("u mage".to_owned()))
        )
        .button(ds::activity::Button {
            label: "Marcide Official github".to_owned(),
            url: "https://github.com/marci1175/marcide".to_owned(),
        })
        .button(ds::activity::Button {
            label: "Marcide Official discord".to_owned(),
            url: "https://discord.gg/7s3VRr4H6j".to_owned(),
        })
        .start_timestamp(SystemTime::now());

    match client.discord.update_activity(rp).await {
        Ok(_) => {},
        Err(err) => println!("{}",err)
    }

    let mut r = String::new();
    let _ = std::io::stdin().read_line(&mut r);

    match client.discord.clear_activity().await {
        Ok(_) => {},
        Err(err) => println!("{}",err)
    }
    


    client.discord.disconnect().await;

    Ok(())
}
