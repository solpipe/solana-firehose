use std::sync::Mutex;

use tonic::transport::{Server, server::Router};

use crate::{
    pipe::utility_server::UtilityServer,
    pipe::broadcast_server::BroadcastServer,
};

use {
    std::{
        sync::Arc,
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
};

/*
let r_addr = grpc_listen_url.parse();
    if r_addr.is_err(){
        return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::ConfigurationError { msg: "no grpc listen address".to_string() }.into()))
    } */


pub fn start_firehose()->Result<Router>{

    let index=Arc::new(Mutex::new(0));
    
    

    let s_1 = FirehosePipe::new(index.clone())?;
    let s_2 = FirehosePipe::new(index.clone())?;

    let utility_service= UtilityServer::new(s_1);
    let broadcast_service=BroadcastServer::new(s_2);
    let router = Server::builder().concurrency_limit_per_connection(64)
        .add_service(utility_service)
        .add_service(broadcast_service);
    
    Ok(router)
}

pub struct FirehosePipe{
    index: Arc<Mutex<u16>>,
}

impl FirehosePipe{
    pub fn new(index: Arc<Mutex<u16>>)->Result<Self>{
        
        return Ok(Self{
            index,
        })
    }
}


pub trait Write{
    fn try_serialize(&self, writer: &mut dyn std::io::Write) -> anchor_lang::Result<()>;
}



