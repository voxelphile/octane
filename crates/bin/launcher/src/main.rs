use std::env;
use std::process;

#[derive(Debug)]
enum Error {
    InvalidArgs = -1,
    #[cfg(debug_assertions)]
    FailedToBuild = -2,
    FailedToRun = -3,
}

fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        Err(Error::InvalidArgs)?
    }

    let mut mode = "release";

    //build the client
    #[cfg(debug_assertions)]
    {
        mode = "debug";

        let mut builder = process::Command::new("sh")
            .arg("-c")
            .arg(format!("cargo build --bin {}", &args[1]))
            .spawn()
            .expect("failed to spawn process");

        let status = builder.wait().expect("failed to wait on build");

        if !status.success() {
            Err(Error::FailedToBuild)?
        }
    }

    let launch = format!("/home/brynn/dev/octane/target/{}/{}", mode, &args[1]);

    //launch the client
    println!("Launching {}.", &args[1]);

    let mut app = process::Command::new("sh")
        .arg("-c")
        .arg(launch)
        .spawn()
        .expect("failed to spawn process");

    let status = app.wait().expect("failed to wait on app");

    if !status.success() {
        Err(Error::FailedToRun)?
    }

    Ok(())
}
