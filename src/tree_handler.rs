use crate::tree::Tree;
use async_std::sync::RwLock;
use async_trait::async_trait;
use log::*;
use nvim_rs::{exttypes::Buffer, runtime::AsyncWrite, Handler, Neovim, Value};
use std::collections::HashMap;
use std::collections::LinkedList;
use std::sync::Arc;

#[derive(Default)]
pub struct TreeHandlerData {
    cfg_map: HashMap<String, Value>,
    trees: HashMap<(i8, Vec<u8>), Tree>,
    treebufs: LinkedList<(i8, Vec<u8>)>, // recently used order
    // buffer: Option<Buffer<<TreeHandler as Handler>::Writer>>,
    buf_count: u32,
}
type TreeHandlerDataPtr = Arc<RwLock<TreeHandlerData>>;

pub struct TreeHandler<W: AsyncWrite + Send + Sync + Unpin + 'static> {
    _phantom: Option<W>, // ugly, but otherwise the compiler will complain, need to workout a more elegant way
    data: TreeHandlerDataPtr,
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
    async fn create_namespace(nvim: Neovim<<Self as Handler>::Writer>) -> i64 {
        let ns_id = nvim.create_namespace("tree_icon").await.unwrap();
        info!("namespace_id for tree_icon: {}", ns_id);
        ns_id
    }

    async fn create_tree(
        data: TreeHandlerDataPtr,
        _nvim: Neovim<<Self as Handler>::Writer>,
        buf: Buffer<<Self as Handler>::Writer>,
        ns_id: i64,
    ) {
        let bufnr = buf.get_value().as_ext().unwrap();
        let bufnr = (bufnr.0, Vec::from(bufnr.1));
        info!("bufnr: {:?}", bufnr);
        let mut tree = Tree::new(bufnr.clone(), ns_id);
        {
            let d = data.read().await;
            tree.config.update(&d.cfg_map);
        }
        {
            let mut d = data.write().await;
            d.trees.insert(bufnr.clone(), tree);
            d.treebufs.push_front(bufnr.clone());
        }
    }

    async fn create_buf(
        data: TreeHandlerDataPtr,
        nvim: Neovim<<Self as Handler>::Writer>,
    ) -> Buffer<<Self as Handler>::Writer> {
        let buf = nvim.create_buf(false, true).await.unwrap();
        info!("new buf created: {:?}", buf.get_value());
        let mut d = data.write().await;
        let buf_num = d.buf_count;
        // TODO: use atomic?
        d.buf_count += 1;
        let buf_name = format!("Tree-{}", buf_num);
        buf.set_name(&buf_name).await.unwrap();
        buf
    }

    async fn start_tree(
        data: TreeHandlerDataPtr,
        nvim: Neovim<<Self as Handler>::Writer>,
        path: String,
    ) {
        info!("start_tree");
        let is_new;
        {
            let d = data.read().await;
            let new_val = match d.cfg_map.get("new") {
                Some(Value::Boolean(v)) => Some(v),
                _ => None,
            };
            is_new = if d.trees.len() < 1 || new_val.is_some() && *new_val.unwrap() {
                /*
                d.resource
                    .insert("start_path".to_owned(), Value::from(path));
                */
                true
            } else {
                false
            };
        }
        if is_new {
            info!("creating new tree");
            let ns_id = Self::create_namespace(nvim.clone()).await;
            let buf = Self::create_buf(data.clone(), nvim.clone()).await;
            Self::create_tree(data, nvim, buf, ns_id).await;
        } else {
        }
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
        info!("Request: {}", name);
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

        match name.as_ref() {
            "_tree_start" => {
                if args.len() <= 0 {
                    return Err(Value::from("Error: path is required for _tree_start"));
                }
                let mut cfg_map = HashMap::new();
                info!("context: {:?}", context);
                for (k, v) in context {
                    let key = match k {
                        Value::String(v) => v.to_string(),
                        _ => return Err(Value::from(format!("Key should be of type string"))),
                    };
                    cfg_map.insert(key, v);
                }
                {
                    let mut d = self.data.write().await;
                    d.cfg_map = cfg_map;
                }

                let path = match &method_args[0] {
                    Value::String(s) => s.as_str().unwrap().to_owned(),
                    _ => return Err(Value::from("Error: path should be string")),
                };
                let data = self.data.clone();
                tokio::spawn(async move {
                    Self::start_tree(data, nvim, path).await;
                });

                Ok(Value::Nil)
            }
            _ => Err(Value::from(format!("Unknown method: {}", name))),
        }
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        info!("Notify: {}", name);
        for arg in args {
            info!("{}", arg);
        }
    }
}
