use crate::errors::ArgError;
use crate::tree::Context;
use crate::tree::Tree;
use async_std::sync::Arc;
use async_std::sync::RwLock;
use async_trait::async_trait;
use futures::io::AsyncWrite;
use log::*;
use nvim_rs::{exttypes::Buffer, Handler, Neovim, Value};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::From;

fn bufnr_val_to_tuple(val: &Value) -> Option<(i8, Vec<u8>)> {
    match val {
        Value::Integer(v) => Some((0, vec![v.as_u64().unwrap() as u8])),
        Value::Ext(v1, v2) => Some((*v1, v2.clone())),
        _ => None,
    }
}

// fn tuple_to_bufnr_val(v: &(i8, Vec<u8>)) -> Value {
//     Value::Ext(v.0.clone(), v.1.clone())
// }

#[derive(Default, Debug)]
pub struct TreeHandlerData {
    // cfg_map: HashMap<String, Value>,
    bufnr_to_tree: HashMap<(i8, Vec<u8>), Tree>,
    tree_bufs: Vec<Value>, // recently used order
    // buffer: Option<Buffer<<TreeHandler as Handler>::Writer>>,
    buf_count: u32,
    prev_bufnr: Option<Value>,
}

type TreeHandlerDataPtr = Arc<RwLock<TreeHandlerData>>;

/// Handling requests and notifications from neovim
pub struct TreeHandler<W: AsyncWrite + Send + Sync + Unpin + 'static> {
    _phantom: std::marker::PhantomData<W>, // ugly, but otherwise the compiler will complain, need to workout a more elegant way
    data: TreeHandlerDataPtr,
}

impl<W: AsyncWrite + Send + Sync + Unpin + 'static> Clone for TreeHandler<W> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(), // the shared state
            _phantom: Default::default(),
        }
    }
}

impl<W: AsyncWrite + Send + Sync + Unpin + 'static> Default for TreeHandler<W> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(TreeHandlerData::default())),
            _phantom: Default::default(),
        }
    }
}

impl<W: AsyncWrite + Send + Sync + Unpin + 'static> TreeHandler<W> {
    async fn create_namespace(
        nvim: &Neovim<<Self as Handler>::Writer>,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let ns_id = nvim.create_namespace("tree_icon").await?;
        Ok(ns_id)
    }

    async fn create_tree(
        data: &mut TreeHandlerData,
        nvim: &Neovim<<Self as Handler>::Writer>,
        bufnr: Value,
        path: &str,
        cfg_map: HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buf = Buffer::new(bufnr.clone(), nvim.clone());
        // new namespace and new buffer for the new tree
        let ns_id = Self::create_namespace(nvim).await?;

        let mut tree = Tree::new(bufnr.clone(), &buf, &nvim, ns_id).await?;
        {
            tree.config.update(&cfg_map)?;
        }

        let start = std::time::Instant::now();
        tree.change_root(path, &nvim).await?;
        info!("change root took: {} secs", start.elapsed().as_secs_f64());

        buf.set_option("buflisted", Value::from(tree.config.listed))
            .await?;

        data.bufnr_to_tree
            .insert(bufnr_val_to_tuple(&bufnr).unwrap(), tree);
        data.tree_bufs.push(bufnr.clone());
        data.prev_bufnr = Some(bufnr.clone());

        // let start = std::time::Instant::now();
        // let nvim = nvim.clone();
        // async_std::task::spawn(async move {
        nvim.execute_lua("tree.resume(...)", vec![bufnr])
            .await
            .unwrap();
        // });
        // info!("resume took: {} secs", start.elapsed().as_secs_f64());
        Ok(())
    }

    /// starts the tree, either create a new one or using the existing one
    async fn start_tree(
        data: &mut TreeHandlerData,
        nvim: &Neovim<<Self as Handler>::Writer>,
        path: String,
        cfg_map: HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bufnr = cfg_map.get("bufnr");

        if let Some(bufnr) = bufnr {
            info!("creating new tree at {}", bufnr);
            // let start = std::time::Instant::now();
            Self::create_tree(data, nvim, bufnr.clone(), &path, cfg_map).await?;
        // info!("Create tree took {} secs", start.elapsed().as_secs_f64());
        } else {
            let bufnr_vals;
            {
                // only a few items, wouldn't be a problem
                let prev_bufnr = match &data.prev_bufnr {
                    Some(nr) => nr,
                    None => return Err(Box::new(ArgError::new("prev_bufnr not defined"))),
                }
                .clone();
                let tree = match data
                    .bufnr_to_tree
                    .get_mut(&bufnr_val_to_tuple(&prev_bufnr).unwrap())
                {
                    Some(t) => t,
                    None => return Err(Box::new(ArgError::new("unknown tree"))),
                };
                tree.config.update(&cfg_map)?;
                data.tree_bufs.retain(|v| v != &prev_bufnr);
                data.tree_bufs.push(prev_bufnr);
                bufnr_vals = Value::Array(data.tree_bufs.iter().rev().cloned().collect());
            }
            nvim.execute_lua("tree.resume(...)", vec![bufnr_vals])
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Sync + Unpin + 'static> Handler for TreeHandler<W> {
    type Writer = W;
    async fn handle_request(
        &self,
        name: String,
        mut args: Vec<Value>,
        nvim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("Request: {}, {:?}", name, args);

        match name.as_ref() {
            "_tree_start" => {
                let vl = match &mut args[0] {
                    Value::Array(v) => v,
                    _ => return Err(Value::from("Error: invalid arg type")),
                };
                let context = match vl.pop() {
                    Some(Value::Map(v)) => v,
                    _ => return Err(Value::from("Error: invalid arg type")),
                };
                let method_args = match vl.pop() {
                    Some(Value::Array(v)) => v,
                    _ => return Err(Value::from("Error: invalid arg type")),
                };
                if args.len() <= 0 {
                    return Err(Value::from("Error: path is required for _tree_start"));
                }
                let mut cfg_map = HashMap::new();
                for (k, v) in context {
                    let key = match k {
                        Value::String(v) => v.into_str().unwrap(),
                        _ => return Err(Value::from(format!("Key should be of type string"))),
                    };
                    cfg_map.insert(key, v);
                }

                let path = match &method_args[0] {
                    Value::String(s) => s.as_str().unwrap().to_owned(),
                    _ => return Err(Value::from("Error: path should be string")),
                };
                info!("path: {}, cfg_map: {:?}", path, cfg_map);
                /*
                tokio::spawn(async move {
                    if let Err(e) = Self::start_tree(data, nvim, path, cfg_map).await {
                        error!("Start tree error: {:?}", e);
                    };
                });
                Ok(Value::Nil)
                */
                let start = std::time::Instant::now();
                {
                    let mut d = self.data.write().await;
                    info!("Wait for lock took {} secs", start.elapsed().as_secs_f64());
                    match Self::start_tree(d.borrow_mut(), &nvim, path, cfg_map).await {
                        Err(e) => Err(Value::from(format!("Error: {:?}", e))),
                        _ => {
                            info!("Start tree took {} secs", start.elapsed().as_secs_f64());
                            Ok(Value::Nil)
                        }
                    }
                }
            }
            "_tree_get_candidate" => {
                let buf = match nvim.get_current_buf().await {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(Value::from(format!("Can't get current buffer: {:?}", e)));
                    }
                };
                let bufnr = match buf.get_value() {
                    Value::Ext(v0, v1) => (*v0, v1.clone()),
                    _ => {
                        return Err(Value::from(format!("Type for current buffer error")));
                    }
                };
                let cursor = match nvim.call_function("line", vec![Value::from(".")]).await {
                    Ok(Value::Integer(v)) => match v.as_u64() {
                        Some(i) => i as usize,
                        None => {
                            return Err(Value::from(format!("Type for current line error")));
                        }
                    },
                    _ => {
                        return Err(Value::from(format!("Type for current line error")));
                    }
                };
                info!("bufnr: {:?}, cursor {}", bufnr, cursor);
                let d = self.data.read().await;
                if let Some(tree) = d.bufnr_to_tree.get(&bufnr) {
                    Ok(Value::from(tree.get_context_value(cursor)))
                } else {
                    Err(Value::from("Can't find view"))
                }
            }
            _ => Err(Value::from(format!("Unknown method: {}", name))),
        }
    }

    async fn handle_notify(
        &self,
        name: String,
        mut args: Vec<Value>,
        neovim: Neovim<Self::Writer>,
    ) {
        info!("Notify {}: {:?}", name, args);
        let vl = std::mem::replace(args.get_mut(0).unwrap(), Value::Nil);
        let mut vl = match vl {
            Value::Array(v) => v,
            _ => {
                error!("Invalid argument type");
                return;
            }
        };
        info!("vl: {:?}", vl);
        if name == "_tree_async_action" && !args.is_empty() {
            if vl.len() != 3 {
                error!("Arg num should be 3 but got {}", vl.len());
            }

            let mut ctx = Context::default();

            // 3rd update context
            match vl.pop().unwrap() {
                Value::Map(context_val) => {
                    for (k, v) in context_val {
                        let key = match k {
                            Value::String(v) => v.into_str().unwrap(),
                            _ => {
                                error!("Key should be of type string");
                                return;
                            }
                        };
                        ctx.update(&key, v);
                    }
                }
                _ => {
                    error!("Context must be of map");
                    return;
                }
            };
            // 2nd
            let act_args = vl.pop().unwrap();

            let action = match vl.pop().unwrap() {
                Value::String(a) => a.into_str().unwrap(),
                _ => {
                    error!("action must be of string type");
                    return;
                }
            };

            info!("async action: {}", action);

            {
                let start = std::time::Instant::now();
                let mut d = self.data.write().await;
                info!(
                    "Waited took {} secs for lock",
                    start.elapsed().as_secs_f64()
                );
                d.prev_bufnr = ctx.prev_bufnr.clone();
                if let Some(bufnr) = ctx.prev_bufnr.clone() {
                    if let Some(tree) = d
                        .bufnr_to_tree
                        .get_mut(&bufnr_val_to_tuple(&bufnr).unwrap())
                    {
                        let start = std::time::Instant::now();
                        tree.action(&neovim, &action, act_args, ctx).await;
                        info!(
                            "Action {} took {} secs",
                            action,
                            start.elapsed().as_secs_f64()
                        );
                    }
                }
            }
        }

        if name == "_tree_async_func" {
            let func_name = args[0].as_str().unwrap();
            if func_name == "paste" {
                let fargs = args[1].as_array().unwrap();
                let pos = fargs[0].as_array().unwrap();
                let src = fargs[1].as_str().unwrap();
                let dest = fargs[2].as_str().unwrap();
                let buf = pos[0].as_u64().unwrap();
                let line = pos[1].as_u64().unwrap();
                {
                    let mut d = self.data.write().await;
                    if let Some(tree) = d
                        .bufnr_to_tree
                        .get_mut(&bufnr_val_to_tuple(&Value::from(buf)).unwrap())
                    {
                        match tree.func_paste(&neovim, line, src, dest).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("paste error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}
