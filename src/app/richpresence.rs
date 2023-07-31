use discord_presence::Client;

pub fn main(file_name : String) {

    let mut drpc = Client::new(1135199303502151711);

    drpc.on_ready(|_ctx| {
        println!("READY!");
    });

    drpc.on_error(|ctx| {
        eprintln!("An error occured, {}", ctx.event);
    });

    let drpc_thread = drpc.start();

    if let Err(why) = drpc.set_activity(|a| {
        a.state("Running examples").assets(|ass| {
            ass.large_image("ferris_wat")
                .large_text(file_name)
                .small_image("rusting")
                .small_text("rusting...")
        })
    }) {
        println!("Failed to set presence: {}", why);
    }

    drpc_thread.join().unwrap()
}