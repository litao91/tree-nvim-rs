use crate::tree::Tree;
use async_trait::async_trait;
use fork::{daemon, Fork};
use log::*;
use nvim_rs::{create, exttypes::Buffer, runtime::Command, Handler, Neovim, Value};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::collections::HashMap;
use std::convert::Into;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::io::WriteHalf;
use tokio::net::UnixStream;
mod column;
mod fs_utils;
mod tree;
mod tree_handler;
mod errors;
use tree_handler::TreeHandler;

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

async fn init_channel<T>(nvim: &Neovim<T>)
where
    T: Sync + Send + Unpin + tokio::io::AsyncWrite,
{
    let chan = nvim.get_api_info().await.unwrap()[0].as_i64().unwrap();
    nvim.set_var("tree#_channel_id", Value::from(chan))
        .await
        .unwrap();
    info!("Set chan to {} done!", chan);
    // file
    let name = format!("tree_{}_0", Into::<u8>::into(column::ColumnType::FILENAME));
    let cmd = format!(
        "hi {} guifg={}",
        &name,
        column::GUI_COLORS[Into::<usize>::into(column::GuiColor::YELLOW)]
    );
    nvim.command(&cmd).await.unwrap();
    // dir
    let name = format!("tree_{}_1", Into::<u8>::into(column::ColumnType::FILENAME));
    let cmd = format!(
        "hi {} guifg={}",
        &name,
        column::GUI_COLORS[Into::<usize>::into(column::GuiColor::BLUE)]
    );
    nvim.command(&cmd).await.unwrap();

    let name = format!("tree_{}", Into::<u8>::into(column::ColumnType::SIZE));
    let cmd = format!(
        "hi {} guifg={}",
        &name,
        column::GUI_COLORS[Into::<usize>::into(column::GuiColor::GREEN)]
    );
    nvim.command(&cmd).await.unwrap();

    let name = format!("tree_{}", Into::<u8>::into(column::ColumnType::TIME));
    let cmd = format!(
        "hi {} guifg={}",
        &name,
        column::GUI_COLORS[Into::<usize>::into(column::GuiColor::BLUE)]
    );
    nvim.command(&cmd).await.unwrap();

    for i in 0..column::ICONS.len() {
        let name = format!("tree_{}_{}", Into::<u8>::into(column::ColumnType::ICON), i);
        let cmd = format!("hi {} guifg={}", name, column::ICONS[i][1]);
        nvim.command(&cmd).await.unwrap();
    }

    for i in 0..column::GUI_COLORS.len() {
        let name = format!("tree_{}_{}", Into::<u8>::into(column::ColumnType::MARK), i);
        let cmd = format!("hi {} guifg={}", &name, column::GUI_COLORS[i]);
        nvim.command(&cmd).await.unwrap();
    }

    for i in 0..column::GIT_INDICATORS.len() {
        let name = format!("tree_{}_{}", Into::<u8>::into(column::ColumnType::GIT), i);
        let cmd = format!("hi {} guifg={}", &name, column::GIT_INDICATORS[i][1]);
        nvim.command(&cmd).await.unwrap();
    }
}

async fn run(args: Vec<String>) {
    assert_eq!(args[1], "--server");
    debug!("args: {:?}", args);
    let server = &args[2];
    let (nvim, io_handler) =
        create::new_unix_socket(server, TreeHandler::<WriteHalf<UnixStream>>::default())
            .await
            .unwrap();
    init_channel(&nvim).await;
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
