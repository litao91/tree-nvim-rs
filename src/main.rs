use async_std;
use backtrace::Backtrace;
use futures::io::AsyncWrite;
use futures::io::WriteHalf;
use log::*;

#[cfg(unix)]
use async_std::os::unix::net::UnixStream;
use nvim_rs::{create::async_std as create, Neovim, Value};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::env;
use std::error::Error;
mod column;
mod errors;
mod tree;
mod tree_handler;
use tree_handler::TreeHandler;

fn init_logging() -> Result<(), Box<dyn Error>> {
    use std::env::VarError;

    let log_level_filter = match env::var("LOG_LEVEL")
        .unwrap_or(String::from("info"))
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

    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filepath)?;

    WriteLogger::init(log_level_filter, config, log_file)?;

    Ok(())
}

fn panic_hook() {
    use std::panic;

    panic::set_hook(Box::new(|p| {
        let backtrace = Backtrace::new();
        error!("panic {:?}\n{:?}", p, backtrace);
    }));
}

/// Initialize the neovim channel
/// It sets up the channel_id and and highlight groups
async fn init_channel<T>(nvim: &Neovim<T>)
where
    T: Sync + Send + Unpin + AsyncWrite,
{
    let chan = nvim.get_api_info().await.unwrap()[0].as_i64().unwrap();
    debug!("setting chan to {}", chan);
    nvim.execute_lua("require('tree').channel_id = ...", vec![Value::from(chan)])
        .await
        .unwrap();
    info!("Set chan to {} done!", chan);

    let mut commands = Vec::new();
    for icon in column::ICONS {
        let name = icon.hl_group_name();
        let color = icon.as_glyph_and_color().1;
        let cmd = format!("hi {} guifg={}", name, color);
        commands.push(Value::from(cmd));
    }

    for color in column::GUI_COLORS {
        let cmd = format!("hi {} guifg={}", color.hl_group_name(), color.color_val(),);
        commands.push(Value::from(cmd));
    }
    nvim.execute_lua("require('tree').run_commands_batch(...)", vec![Value::from(commands)]).await.unwrap();
}

async fn run(args: Vec<String>) {
    debug!("args: {:?}", args);
    let server = args[1].clone();
    // create the neovim session with TreeHandler
    let (nvim, io_handler) = create::new_unix_socket(
        server,
        TreeHandler::<WriteHalf<UnixStream>>::default(),
    )
    .await
    .unwrap();
    // set tree#_channel_id
    init_channel(&nvim).await;

    match io_handler.await {
        Err(err) => {
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
        Ok(()) => {}
    }
}

#[async_std::main]
async fn main() {
    let _ = init_logging();
    panic_hook();
    let args: Vec<String> = env::args().collect();
    debug!("No fork");
    run(args).await;
    debug!("Done!");
}
