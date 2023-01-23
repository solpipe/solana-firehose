use std::{sync::{Mutex, RwLock}, fs::File};
use memmap::MmapMut;
use solana_geyser_plugin_interface::geyser_plugin_interface::{GeyserPluginError, ReplicaAccountInfoV2};
use solana_sdk::{ signature::Signature, transaction::SanitizedTransaction};
use sprintf::sprintf;

use crate::geyser_plugin_firehose::GeyserPluginFirehoseError;
use {
    std::{
        sync::Arc,
        fs::{
            create_dir,read_dir,
            rename,
            OpenOptions,
        },
        io::{Seek, SeekFrom, Write},
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
    bincode::serialize,
};

pub struct FirehosePipeConfiguration{
    directory: String,
    spots: usize,
    max_account_data_size: usize,
}
pub struct FirehosePipe{
    config: FirehosePipeConfiguration,
    slots: Arc<RwLock<Vec<Arc<RwLock<TransactionSpot>>>>>,
}



impl FirehosePipe{


    
    pub fn new(config: FirehosePipeConfiguration)->Result<Self>{
        
        {
            let list=Vec::new();
            let d_write=sprintf!("%s/write",config.directory).unwrap();
            list.push(d_write);
            let d_read=sprintf!("%s/read",config.directory).unwrap();
            list.push(d_read);
            let d_done=sprintf!("%s/done",config.directory).unwrap();
            list.push(d_done);
            
            for x in list.iter(){
                create_dir(x)?;
            }
        }
        
        // create open, writing, reading directories
        let mut spot_list = Vec::new();
        for id in 0..config.spots{
            let tx_spot = TransactionSpot::new(config.directory,id)?;
            spot_list.push(Arc::new(RwLock::new(tx_spot)));
        }

        return Ok(Self{
            config,
            slots: Arc::new(RwLock::new(spot_list)),
        })
    }

    pub fn shutdown(&mut self){

    }

    fn slot_spot_add(&self)->Result<()>{

        Ok(())
    }
    
}





#[derive(Debug, PartialEq, Eq)]
pub enum SpotLocation{
    READ=0,WRITE=1,PENDING=2,
}


pub struct TransactionSpot{
    id: usize,
    directory: String,
    loc: SpotLocation,
    f: File,
    data: MmapMut,
    index: usize,
    txn_id: Option<Signature>,
}
const SIZE: u64 = 1024 * 1024;

impl TransactionSpot{
    pub fn new(directory: String,id: usize)->Result<Self>{
        
        let loc = SpotLocation::PENDING;
        let fp = TransactionSpot::location_fp(directory,loc);
        
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(fp)?;
        f.seek(SeekFrom::Start(SIZE))?;
        f.write_all(&[0])?;
        f.seek(SeekFrom::Start(0))?;
        let mut data = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&f)?
        };
        Ok(Self {
            id, directory,loc,f,data,txn_id:None,index:0,
        })
    }

    pub fn location(&self)->SpotLocation{
        return self.loc;
    }

    fn location_change(&mut self,location:SpotLocation)->Result<()>{
        if self.location()==location{
            return Ok(())
        }
        let old_fp = TransactionSpot::location_fp(self.directory,self.location());
        let new_fp = TransactionSpot::location_fp(self.directory,location);
        rename(old_fp,new_fp)?;
        self.loc = location;
        Ok(())
    }

    fn location_fp(directory: String, location: SpotLocation)->String{
        let ans;
        match location{
            SpotLocation::READ => {
                ans=sprintf!("%s/read",directory).unwrap();
            },
            SpotLocation::WRITE => {
                ans=sprintf!("%s/write",directory).unwrap();
            },
            SpotLocation::PENDING => {
                ans=sprintf!("%s/pending",directory).unwrap();
            },
        }
        return ans;
    }

    pub fn txn_set(&mut self,signature: &Signature)->Result<()>{
        if self.location()!=SpotLocation::PENDING{
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::NotWriting.into()))
        }

        if self.txn_id.is_some(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::TxnAlreadySet.into()))
        }
        if 0<self.index{
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::TxnAlreadySet.into()))
        }
        self.location_change(SpotLocation::WRITE)?;

        self.txn_id = Some(signature.clone());
        
        let r_data = serialize(signature);
        if r_data.is_err(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::FailedToSerialize.into()))
        }
        
        let sig_data = r_data.unwrap();
        let sig_len = sig_data.len() as u32;
        self.data[self.index..self.index+4].copy_from_slice(&sig_len.to_be_bytes().to_vec());
        self.index+=4;
        self.data[self.index..self.index+sig_data.len()].copy_from_slice(&sig_data);
        self.index+=sig_data.len();
        
        Ok(())
    }
    

    pub fn save_account(&mut self,account: ReplicaAccountInfoV2)->Result<()>{
        if self.location()!=SpotLocation::WRITE{
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::NotWriting.into()))
        }


        let s_u_64: usize = 8;
        let s_u_32: usize = 4;
        let s_bool: usize = 1;

        let old_index = self.index;
        self.index+=s_u_32; // u32 for data size

        // pub pubkey: &'a [u8],
        self.data[self.index..self.index+account.pubkey.len()].copy_from_slice(account.pubkey);
        self.index+=account.pubkey.len();

        //pub lamports: u64,
        self.data[self.index..self.index+s_u_64].copy_from_slice(&account.lamports.to_be_bytes().to_vec());
        self.index+=s_u_64;

        //pub owner: &'a [u8],
        self.data[self.index..self.index+account.owner.len()].copy_from_slice(account.owner);
        self.index+=account.owner.len();

        //pub executable: bool,
        if account.executable{
            self.data[self.index]=1;
        }else{
            self.data[self.index]=0;
        }
        self.index+=s_bool;

        //pub rent_epoch: u64,
        self.data[self.index..self.index+s_u_64].copy_from_slice(&account.rent_epoch.to_be_bytes().to_vec());
        self.index+=s_u_64;

        //pub data: &'a [u8],
        self.data[self.index..self.index+account.data.len()].copy_from_slice(account.data);
        self.index+=account.data.len();

        //pub write_version: u64,
        self.data[self.index..self.index+s_u_64].copy_from_slice(&account.write_version.to_be_bytes().to_vec());
        self.index+=s_u_64;
        
        // prepend size of account that was written
        let s = (self.index - old_index - s_u_32) as u32;
        self.data[old_index..old_index+s_u_32].copy_from_slice(&s.to_be_bytes().to_vec());
        self.f.flush()?;
        Ok(())
    }

    // the reader is now able to read this transaction spot
    // when the read is done, the reader will move this file to the pending directory
    pub fn mark_read(&mut self)->Result<()>{
        if self.location()!=SpotLocation::WRITE{
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::NotWriting.into()))
        }
        self.location_change(SpotLocation::READ)?;

        Ok(())
    }

    pub fn shutdown(&mut self){
        self.f.flush();
    }
}




