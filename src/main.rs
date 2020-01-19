use crate::tree::Tree;
use async_trait::async_trait;
use futures::io::{AsyncReadExt, WriteHalf};
use async_std;
use backtrace::Backtrace;
use fork::{daemon, Fork};
use futures::io::AsyncWrite;
use log::*;

#[cfg(unix)]
use async_std::os::unix::net::UnixStream;
use nvim_rs::{
    create::async_std as create,
    Neovim, Value,
};
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
    use std::fs::File;

    let log_level_filter = match env::var("LOG_LEVEL")
        .unwrap_or(String::from("error"))
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

fn panic_hook() {
    use std::panic;

    panic::set_hook(Box::new(|p| {
        let backtrace = Backtrace::new();
        error!("panic {:?}\n{:?}", p, backtrace);
    }));
}

async fn init_channel<T>(nvim: &Neovim<T>)
where
    T: Sync + Send + Unpin + AsyncWrite,
{
    let chan = nvim.get_api_info().await.unwrap()[0].as_i64().unwrap();
    nvim.set_var("tree#_channel_id", Value::from(chan))
        .await
        .unwrap();
    info!("Set chan to {} done!", chan);

    for icon in column::ICONS {
        let name = icon.hl_group_name();
        let color = icon.as_glyph_and_color().1;
        let cmd = format!("hi {} guifg={}", name, color);
        nvim.command(&cmd).await.unwrap();
    }

    for color in column::GUI_COLORS {
        let cmd = format!("hi {} guifg={}", color.hl_group_name(), color.color_val(),);
        nvim.command(&cmd).await.unwrap();
    }
}

async fn run(args: Vec<String>) {
    debug!("args: {:?}", args);
    let mut server = None;
    for i in 0..args.len() {
        if args[i] == "--server" {
            server = args.get(i + 1);
        }
    }
    let (nvim, io_handler) = create::new_unix_socket(
        server.unwrap(),
        TreeHandler::<WriteHalf<UnixStream>>::default(),
    )
    .await
    .unwrap();
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
    init_logging().unwrap();
    panic_hook();
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
        unimplemented!();
    }
}
