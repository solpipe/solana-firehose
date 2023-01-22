use std::sync::Mutex;

use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;
use tonic::transport::{Server, server::Router};
use tokio::sync::mpsc::{self, channel};

use crate::{
    pipe::utility_server::UtilityServer,
    pipe::broadcast_server::BroadcastServer, manager::Manager, geyser_plugin_firehose::GeyserPluginFirehoseError,
};

use {
    std::{
        sync::Arc,
        
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
    tokio::runtime::Runtime,
};




pub fn start_firehose(grpc_listen_url: String, rt: Arc<Mutex<Runtime>>)->Result<Manager>{

    let index=Arc::new(Mutex::new(0));
    let (tx, mut rx)=mpsc::channel::<i32>(1);
    
    let mgr=Manager::new(tx);
    let s_1 = FirehosePipe::new(
        index.clone(),
        mgr.clone(),
    )?;
    let s_2 = FirehosePipe::new(
        index.clone(),
        mgr.clone(),
    )?;

    let utility_service= UtilityServer::new(s_1);
    let broadcast_service=BroadcastServer::new(s_2);
    let router = Server::builder().concurrency_limit_per_connection(64)
        .add_service(utility_service)
        .add_service(broadcast_service)
        .serve_with_shutdown(grpc_listen_url.parse().unwrap(), async move {
            let _x = rx.recv().await;
        });
    {
        let rt2 = rt.clone();
        let x = rt2.as_ref().lock();
        if x.is_err(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.into()));
        }
        x.unwrap().spawn(router);
    }
    
    Ok(mgr)
}

pub struct FirehosePipe{
    index: Arc<Mutex<u16>>,
    mgr: Manager,
}

impl FirehosePipe{
    pub fn new(index: Arc<Mutex<u16>>,mgr: Manager)->Result<Self>{
        
        return Ok(Self{
            mgr,
            index,
        })
    }
}





