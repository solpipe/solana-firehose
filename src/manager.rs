use std::{sync::Mutex, clone, thread::sleep, time::Duration};

use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;
use tokio::sync::{mpsc::{self, channel, Receiver, Sender}, RwLock, broadcast};

use crate::{geyser_plugin_firehose::GeyserPluginFirehoseError, pipe::{Account, self}};

use {
    std::{
        sync::Arc,
        
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
    tokio::runtime::Runtime,
};


pub trait SerializableObject{
    fn try_serialize(&self, writer: &mut dyn std::io::Write) -> anchor_lang::Result<()>;
}


const SIZE_SPOT: usize = 50;
#[derive(Clone)]
pub struct Manager{
    pub is_shutdown: Arc<RwLock<bool>>,
    pub spots: Arc<RwLock<Vec<Arc<RwLock<Spot>>>>>,
    pub tx_shutdown: Arc<Mutex<Sender<i32>>>,
    update_c: broadcast::Sender<i32>,
}

impl Manager{

    pub fn new(tx_shutdown: Sender<i32>)->Self{
        let mut spots=Vec::new();
        for i in 0..SIZE_SPOT-1{
            spots.push(Arc::new(RwLock::new(Spot::new(i as i32))));
        }
        let (tx, mut _rx1) = broadcast::channel(16);

        Self{
            update_c: tx,
            is_shutdown:Arc::new(RwLock::new(false)),
            tx_shutdown:Arc::new(Mutex::new(tx_shutdown)),
            spots:Arc::new(RwLock::new(spots)),
        }
    }
    
    pub fn is_closed(&self)->bool{
        let r = self.is_shutdown.as_ref().try_read();
        if r.is_err(){
            return false
        }
        let ans = r.unwrap();
        return *ans
    }

    pub fn shutdown(&self){
        for _i in 0..20{
            let w = self.is_shutdown.as_ref().try_write();
            if w.is_err(){
                sleep(Duration::from_secs(30));   
            } else {
                let mut x = w.unwrap();
                *x=true;
                break
            }
        }
    }
    
    pub fn account_insert(&self,account: &Account)->Result<()>{
        
        let r = self.spots.as_ref().try_read();
        if r.is_err(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.into()))
        }
        let list= r.unwrap();
        let mut iter = list.iter();
        
        while let Some(r_spot) = iter.next(){
            let c = r_spot.as_ref().try_write();
            if c.is_err(){
                continue
            }
            let mut spot = c.unwrap();
            if spot.write(account.clone(),self.update_c.clone()){
                break
            }
        }
        Ok(())
    }

    pub async fn account_read(&self,i: i32)->Result<pipe::Account>{
        let r = self.spots.as_ref().read().await;
        
        let list = r;
        match list.get(i as usize){
            Some(r_spot) => {
                let spot = r_spot.as_ref().read().await;
                match &spot.buf{
                    Some(a) => {
                        Ok(a.clone())
                    },
                    None => {
                        Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.into()))        
                    },
                }
            },
            None => {
                Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.into()))
            },
        }
        
    }
    

    pub fn subscribe(&self)->broadcast::Receiver<i32>{
        return self.update_c.subscribe()
    }
}

const COUNTER_IS_OPEN: i32 = -2;
const COUNTER_IS_WRITING: i32 = -1;
const COUNTER_IS_READING: i32 = 0;

#[derive(Clone)]
pub struct Spot{
    pub id: i32, 
    counter: i32, // how many people are copying
    buf: Option<Account>, // update this
}

impl Spot{
    pub(crate) fn new(id: i32)->Self{
        Self{
            id,
            counter:COUNTER_IS_OPEN,
            buf:None,
        }
    }
    pub(crate) fn write(&mut self,account: Account,update_c: broadcast::Sender<i32>)->bool{
        
        if self.counter==COUNTER_IS_OPEN{
            if update_c.send(self.id).is_err(){
                self.counter=COUNTER_IS_READING;
                self.buf=Some(account);
                return true
            }
        }
        return false
        
    }
    
/*
    pub(crate) fn read(&self)->Result<Arc<RwLock<[u8;SIZE_SPOT_BUFFER]>>>{
        
        let r_counter=self.counter.lock();
        if r_counter.is_err(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.into()));
        }
        let mut counter=r_counter.unwrap();
        *counter+=1;

        Ok(self.buf.clone())
    }

    pub fn mark_read_done(&self){
        let r_counter=self.counter.as_ref().lock();
        if r_counter.is_err(){
            return
        }
        let mut counter=r_counter.unwrap();
        if counter.is_positive(){
            *counter-=1;
        }
    }
    */

}


pub struct Job{
    pub spot_id: i32,
    pub buf: Arc<Vec<u8>>,
    pub done_c: mpsc::Sender<i32>, // send the spot id back
}



pub struct SubscriberReceiver{

    pub account_c: mpsc::Receiver<i32>,
    pub tx_c: mpsc::Receiver<i32>,
}

struct SubscriberSender{
    pub account_c: Arc<Mutex<mpsc::Sender<i32>>>,
    pub tx_c: Arc<Mutex<mpsc::Sender<i32>>>,
}