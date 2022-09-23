use claves::{deinit, init};

fn main() {
    let receiver = init();

    println!("Initialized");

    for _ in 0..30 {
        dbg!(receiver.recv().unwrap());
    }

    println!("Received");

    deinit();

    println!("Deinitialized");
}
