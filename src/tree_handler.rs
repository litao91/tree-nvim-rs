use crate::errors::ArgError;
use crate::tree::Context;
use crate::tree::Tree;
use async_std::sync::RwLock;
use async_trait::async_trait;
use futures::io::AsyncWrite;
use log::*;
use nvim_rs::{exttypes::Buffer, Handler, Neovim, Value};
use std::collections::HashMap;
use std::convert::From;
use async_std::sync::Arc;

#[derive(Default, Debug)]
pub struct TreeHandlerData {
    // cfg_map: HashMap<String, Value>,
    trees: HashMap<(i8, Vec<u8>), Tree>,
    treebufs: Vec<(i8, Vec<u8>)>, // recently used order
    // buffer: Option<Buffer<<TreeHandler as Handler>::Writer>>,
    buf_count: u32,
    prev_bufnr: Option<(i8, Vec<u8>)>,
}

type TreeHandlerDataPtr = Arc<RwLock<TreeHandlerData>>;

pub struct TreeHandler<W: AsyncWrite + Send + Sync + Unpin + 'static> {
    _phantom: Option<W>, // ugly, but otherwise the compiler will complain, need to workout a more elegant way
    data: TreeHandlerDataPtr,
}

impl<W: AsyncWrite + Send + Sync + Unpin + 'static> Clone for TreeHandler<W> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
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
        nvim: Neovim<<Self as Handler>::Writer>,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let ns_id = nvim.create_namespace("tree_icon").await?;
        Ok(ns_id)
    }

    async fn create_tree(
        data: TreeHandlerDataPtr,
        nvim: Neovim<<Self as Handler>::Writer>,
        buf: Buffer<<Self as Handler>::Writer>,
        ns_id: i64,
        path: &str,
        cfg_map: HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bufnr = buf.get_value().as_ext().unwrap();
        let bufnr = (bufnr.0, Vec::from(bufnr.1));
        let mut tree = Tree::new(bufnr.clone(), &buf, &nvim, ns_id).await?;
        {
            tree.config.update(&cfg_map)?;
        }
        let start = std::time::Instant::now();
        tree.change_root(path, &nvim).await?;
        info!("change root took: {} secs", start.elapsed().as_secs_f64());

        let tree_cfg = tree.config.get_cfg_map();
        {
            let mut d = data.write().await;
            d.trees.insert(bufnr.clone(), tree);
            d.treebufs.push(bufnr.clone());
            d.prev_bufnr = Some(bufnr.clone());
        }
        // let start = std::time::Instant::now();
        let args = vec![Value::Ext(bufnr.0, bufnr.1), tree_cfg];
        async_std::task::spawn(async move {
            nvim.execute_lua("resume(...)", args).await.unwrap();
        });
        // info!("resume took: {} secs", start.elapsed().as_secs_f64());
        Ok(())
    }

    async fn create_buf(
        data: TreeHandlerDataPtr,
        nvim: Neovim<<Self as Handler>::Writer>,
    ) -> Result<Buffer<<Self as Handler>::Writer>, Box<dyn std::error::Error>> {
        let buf = nvim.create_buf(false, true).await.unwrap();
        info!("new buf created: {:?}", buf.get_value());
        let buf_num;
        {
            let mut d = data.write().await;
            buf_num = d.buf_count;
            d.buf_count += 1;
        }
        let buf_name = format!("Tree-{}", buf_num);
        buf.set_name(&buf_name).await?;
        Ok(buf)
    }

    async fn start_tree(
        data: TreeHandlerDataPtr,
        nvim: Neovim<<Self as Handler>::Writer>,
        path: String,
        cfg_map: HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("start_tree");
        let is_new;
        {
            let d = data.read().await;
            let new_val = match cfg_map.get("new") {
                Some(Value::Boolean(v)) => Some(v),
                _ => None,
            };
            is_new = if d.trees.len() < 1 || new_val.is_some() && *new_val.unwrap() {
                true
            } else {
                false
            };
        }
        if is_new {
            info!("creating new tree");
            let ns_id = Self::create_namespace(nvim.clone()).await?;
            let buf = Self::create_buf(data.clone(), nvim.clone()).await?;
            // let start = std::time::Instant::now();
            Self::create_tree(data, nvim, buf, ns_id, &path, cfg_map).await?;
        // info!("Create tree took {} secs", start.elapsed().as_secs_f64());
        } else {
            let bufnr_vals;
            let tree_cfg;
            {
                let mut d = data.write().await;
                // only a few items, wouldn't be a problem
                let prev_bufnr = match &d.prev_bufnr {
                    Some(nr) => nr,
                    None => return Err(Box::new(ArgError::new("prev_bufnr not defined"))),
                }
                .clone();
                let tree = match d.trees.get_mut(&prev_bufnr) {
                    Some(t) => t,
                    None => return Err(Box::new(ArgError::new("unknown tree"))),
                };
                tree.config.update(&cfg_map)?;
                tree_cfg = tree.config.get_cfg_map();
                d.treebufs.retain(|v| v != &prev_bufnr);
                d.treebufs.push(prev_bufnr);
                bufnr_vals = Value::Array(
                    d.treebufs
                        .iter()
                        .rev()
                        .cloned()
                        .map(|v| Value::Ext(v.0, v.1))
                        .collect(),
                );
            }
            nvim.command("lua require('tree')").await?;
            nvim.execute_lua("resume(...)", vec![bufnr_vals, tree_cfg])
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
                let data = self.data.clone();
                /*
                tokio::spawn(async move {
                    if let Err(e) = Self::start_tree(data, nvim, path, cfg_map).await {
                        error!("Start tree error: {:?}", e);
                    };
                });
                Ok(Value::Nil)
                */
                let start = std::time::Instant::now();
                match Self::start_tree(data, nvim, path, cfg_map).await {
                    Err(e) => Err(Value::from(format!("Error: {:?}", e))),
                    _ => {
                        info!("Start tree took {} secs", start.elapsed().as_secs_f64());
                        Ok(Value::Nil)
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
                if let Some(tree) = d.trees.get(&bufnr) {
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
                    if let Some(tree) = d.trees.get_mut(&bufnr) {
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
    }
}
