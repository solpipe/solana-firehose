use std::sync::Mutex;

use anchor_lang::{AccountDeserialize, AnchorDeserialize};
use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;

use crate::geyser_plugin_firehose::GeyserPluginFirehoseError;

use {
    libc::{
        close, ftruncate, memcpy, mmap, shm_open, strncpy,
        MAP_SHARED, O_RDWR, O_CREAT, PROT_WRITE, S_IRUSR, S_IWUSR,
        c_char, c_void, off_t, size_t,
    },
    std::{

        sync::Arc,
        fs::File, io::Read,
        env, ptr, str,
        process::Command,
        error::Error,
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
};

const STORAGE_ID   : *const c_char = b"MY_MEM_ID\0".as_ptr() as *const c_char;
const STORAGE_SIZE : size_t        = 128;

pub struct FirehosePipe{
    directory: String,
    fd: Arc<i32>,
    index: Arc<Mutex<u16>>,
    size: u16,
    //addr: Arc<*mut c_void>,
}

impl FirehosePipe{
    pub fn new(directory: String, size: u16)->Result<Self>{
        let fd   =  unsafe {
            let fd = shm_open(STORAGE_ID, O_RDWR | O_CREAT, (S_IRUSR | S_IWUSR).try_into().unwrap());
            fd
        };

        return Ok(Self{
            directory,
            fd: Arc::new(fd),
            index:Arc::new(Mutex::new(0)),
            size,
        })
    }

    fn increment<'a>(&'a self)->Result<u16>{
        let r_l=self.index.lock();
        if r_l.is_err(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::LockNotAcquired.try_into().unwrap()))
        }
        let i = &mut *r_l.unwrap();
        let j = *i;
        *i=(*i+1)%self.size;
        return Ok(j)
    }

    fn offset(i: u16)->off_t{
        return i as off_t
    }

    pub fn write<'a>(&'a self, data: &'a dyn Write)->Result<()>{
        let i = self.increment()?;
        
        let addr = unsafe{
            let null = ptr::null_mut();
            let fd = *self.fd;
            let addr = mmap(null, STORAGE_SIZE, PROT_WRITE, MAP_SHARED, fd, FirehosePipe::offset(i));
            addr
        };

        
        

        //let pdata = data.as_ptr() as *const c_void;
        unsafe {
            let mut r_cursor = std::io::Cursor::new(addr);
            data.try_serialize(&mut r_cursor as &mut [u8])?;
            //memcpy(addr, pdata, data.len());
        }        

        Ok(())
    }

}


pub trait Write{
    fn try_serialize(&self, writer: &mut dyn std::io::Write) -> anchor_lang::Result<()>;
}