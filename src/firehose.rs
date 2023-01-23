use std::sync::Mutex;

use solana_geyser_plugin_interface::geyser_plugin_interface::{GeyserPluginError, ReplicaAccountInfoV2};
use solana_sdk::{ signature::Signature, transaction::SanitizedTransaction};
use std::fs;

use {
    std::{
        sync::Arc,
        
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
};

pub struct FirehosePipeConfiguration{
    directory: String,
    spots: usize,
    max_account_data_size: usize,
}
pub struct FirehosePipe{
    config: FirehosePipeConfiguration,
}

impl FirehosePipe{
    fn spot_fp(i: usize)->String{
        fmt.sprintf()
    }
    pub fn new(config: FirehosePipeConfiguration)->Result<Self>{
        let _x = fs::read_dir(config.directory)?;

        for i in 0..config.spots - 1{

        }

        return Ok(Self{
            config,
        })
    }

    pub fn shutdown(&mut self){

    }
}







pub struct SlotSpot{
    id: usize,
}

impl SlotSpot{
    pub fn new(directory: String)->Result<()>{

    }
    fn get_tx_spot<'a>(&self,signature: Option<&'a Signature>)->Result<TransactionSpot>{
        Ok(())
    }
}

pub struct TransactionSpot{

}

impl TransactionSpot{
    fn save_account(&self,account: ReplicaAccountInfoV2)->Result<()>{
        Ok(())
    }
}