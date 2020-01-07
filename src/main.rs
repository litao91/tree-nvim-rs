mod tree;
use async_trait::async_trait;
use fork::{daemon, Fork};
use log::*;
use nvim_rs::{
    create, neovim_api, neovim_api_manual,
    runtime::{ChildStdin, Command, Stdout},
    Handler, Neovim, Value,
};
use simplelog::{Config, ConfigBuilder, Level, LevelFilter, WriteLogger};
use std::env;
use std::error::Error;
use tokio::net::UnixStream;

struct TreeHandler {}

#[async_trait]
impl Handler for TreeHandler {
    type Writer = UnixStream;
    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("Request: {}", name);
        for arg in args {
            info!("{}", arg);
        }
        Ok(Value::Nil)
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, neovim: Neovim<Self::Writer>) {
        info!("Notify: {}", name);
        for arg in args {
            info!("{}", arg);
        }
    }
}

fn init_logging() -> Result<(), Box<dyn Error>> {
    use std::env::VarError;
    use std::fs::File;

    let log_level_filter = match env::var("LOG_LEVEL")
        .unwrap_or(String::from("trace"))
        .to_lowercase()
        .as_ref()
    {
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "info" => LevelFilter::Info,
        "off" => LevelFilter::Off,
        "trace" => LevelFilter::Trace,
        "warn" => LevelFilter::Warn,
        _ => LevelFilter::Off,
    };

    let config = ConfigBuilder::new()
        .set_max_level(LevelFilter::Info)
        .build();

    let filepath = match env::var("LOG_FILE") {
        Err(err) => match err {
            VarError::NotPresent => return Ok(()),
            e @ VarError::NotUnicode(_) => {
                return Err(Box::new(e));
            }
        },
        Ok(path) => path.to_owned(),
    };

    let log_file = File::create(filepath)?;

    WriteLogger::init(log_level_filter, config, log_file).unwrap();

    Ok(())
}

async fn run(args: Vec<String>) {
    assert_eq!(args[1], "--server");
    debug!("args: {:?}", args);
    let server = &args[2];
    let (nvim, io_handler) = create::new_unix_socket(server, TreeHandler {})
        .await
        .unwrap();
    nvim.set_var("tree#_channel_id", Value::from(100)).await.unwrap();

    match io_handler.await {
        Err(joinerr) => error!("Error joining IO loop '{}'", joinerr),
        Ok(Err(err)) => {
            if !err.is_reader_error() {
                // One last try, since there wasn't an error with writing to the stream
                nvim.err_writeln(&format!("Error: '{}'", err))
                    .await
                    .unwrap_or_else(|e| {
                        // We could inspect this error to see what was happening, and maybe
                        // retry, but at this point it's probably best to assume the worst
                        // and print a friendly and supportive message to our
                        // users
                        error!("Well, dang... '{}'", e);
                    });
            }

            if !err.is_channel_closed() {
                // Closed channel usually means neovim quit itself, or this plugin was
                // told to quit by closing the channel, so it's not always an error
                // condition.
                error!("Error: '{}'", err);
            }
        }
        Ok(Ok(())) => {}
    }
}

#[tokio::main]
async fn main() {
    init_logging().unwrap();
    let mut args: Vec<String> = env::args().collect();
    let mut nofork = false;
    for arg in &args {
        if arg == "--nofork" {
            nofork = true;
        }
    }
    if nofork {
        debug!("No fork");
        run(args).await;
        debug!("Done!");
    } else {
        debug!("demonalizing");
        if let Ok(Fork::Child) = daemon(false, false) {
            args.push("--nofork".to_string());
            let _ = Command::new(&args[0]).args(&args[1..]).spawn().unwrap();
        }
        info!("Return from parent!");
    }
}
