use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPluginError, SlotStatus, ReplicaAccountInfoVersions, ReplicaTransactionInfoVersions,
};
use solana_transaction_status::TransactionStatusMeta;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use {
    std::{
        fs::remove_dir_all,
        io::Write,
        os::unix::net::UnixStream,
        net::Shutdown,
    },
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        Result,
    },
};
use crate::geyser_plugin_firehose::GeyserPluginFirehoseError;

fn connect(socket_path: &String) -> Result<UnixStream> {
    return match UnixStream::connect(socket_path.clone()){
        Ok(x) => Ok(x),
        Err(e) => {
            eprintln!("unix connect (path={}) error: {}",socket_path,e.to_string());
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::ConnectionFailed.into()))
        },
    };
}

#[derive(Clone)]
pub struct FirehosePipeConfiguration{
    pub account_path: String,
    pub slot_path: String,
    pub transaction_path: String,
    //pub max_account_data_size: usize,
}


#[derive(Clone)]
pub struct FirehosePipe{
    config: FirehosePipeConfiguration,
}

//#[derive(Default)]


impl FirehosePipe{   
    
    pub fn new(config: FirehosePipeConfiguration)->Result<Self>{        
        // create open, writing, reading directories
        return Ok(Self{
            config,
        })
    }

    pub fn on_slot(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    )->Result<()>{
        let mut stream = connect(&self.config.slot_path)?;

        stream.write_u64::<BigEndian>(slot)?;
        
        match parent{
            Some(x) => {
                stream.write_u64::<BigEndian>(x)?;
            },
            None => {
                stream.write_u64::<BigEndian>(0)?;
            },
        }
        match status{
            SlotStatus::Processed => {
                stream.write_u8(0)?;
            },
            SlotStatus::Confirmed => {
                stream.write_u8(1)?;
            },
            SlotStatus::Rooted => {
                stream.write_u8(2)?;
            },
        }

        stream.flush()?;
        stream.shutdown(Shutdown::Write)?;

        let s = stream.read_u8()?;
        if 0 < s{
            Ok(())
        } else {
            abort();
            //return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::IOFailure.into()))
        }
    }


    pub fn shutdown(&mut self){
        let _r=remove_dir_all(self.config.account_path.clone());
    }

    pub fn on_account(
        &self,
        account_in: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    )->Result<()>{
        let account ;
        match account_in{
            ReplicaAccountInfoVersions::V0_0_1(_) => {
                return Ok(())
            },
            ReplicaAccountInfoVersions::V0_0_2(x) => {
                account=x;
            },
        };

        let sig = match account.txn_signature{
            Some(x) => x,
            None => {
                return Ok(())
            },
        };

        let mut stream = connect(&self.config.account_path)?;
        stream.write_u64::<BigEndian>(slot)?;
        if is_startup{
            stream.write_u8(1)?;
        }else{
            stream.write_u8(0)?;
        }

        // pub pubkey: &'a [u8],
        stream.write_all(sig.as_ref())?;

        stream.write_all(account.pubkey)?;

        //pub lamports: u64,
        stream.write_u64::<BigEndian>(account.lamports)?;

        //pub owner: &'a [u8],
        stream.write_all(account.owner)?;

        //pub executable: bool,
        if account.executable{
            stream.write_u8(1)?;
        }else{
            stream.write_u8(0)?;
        }
        //pub rent_epoch: u64,
        stream.write_u64::<BigEndian>(account.rent_epoch)?;

        //pub write_version: u64,
        stream.write_u64::<BigEndian>(account.write_version)?;

        //pub data: &'a [u8], // unknown data size
        stream.write_u64::<BigEndian>(account.data.len() as u64)?;
        stream.write_all(account.data)?;

        stream.flush()?;
        stream.shutdown(Shutdown::Write)?;

        let s = stream.read_u8()?;
        if 0 < s{
            Ok(())
        } else {
            abort();
            //return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::IOFailure.into()))
        }
    }

    pub fn on_transction(
        &self,
        transaction_info: ReplicaTransactionInfoVersions,
        slot: u64,
    )->Result<()>{
        let tx = match transaction_info{
            ReplicaTransactionInfoVersions::V0_0_1(_) => {
                return Ok(())
            },
            ReplicaTransactionInfoVersions::V0_0_2(x) => x,
        };
        
        let mut stream = connect(&self.config.transaction_path)?;
        stream.write_u64::<BigEndian>(slot)?;
        
        // The first signature of the transaction, used for identifying the transaction.
        //    pub signature: &'a Signature,
        stream.write_all(tx.signature.as_ref())?;
        
        
        // Indicates if the transaction is a simple vote transaction.
        //  pub is_vote: bool,
        if tx.is_vote{
            stream.write_u8(1)?;
        } else{
            stream.write_u8(0)?;
        }
        // The transaction's index in the block
        // pub index: usize,
        stream.write_u64::<BigEndian>(tx.index as u64)?;
        // The sanitized transaction.
        //  pub transaction: &'a SanitizedTransaction,
        //self.sanitizedtx(&stream, tx.transaction)?;

        // Metadata of the transaction status.
        // pub transaction_status_meta: &'a TransactionStatusMeta,
        self.metatx(&stream, tx.transaction_status_meta)?;
        stream.flush()?;
        stream.shutdown(Shutdown::Write)?;
        
        let s = stream.read_u8()?;
        if 0 < s{
            Ok(())
        } else {
            abort();
            //return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::IOFailure.into()))
        }
    }
    

    fn metatx<'b: 'a,'a>(
        &'a self,
        mut stream: &'b UnixStream,
        meta: &TransactionStatusMeta,
    )->Result<()>{

        //    pub status: TransactionResult<()>,
        if !meta.status.is_err(){
            stream.write_u8(1)?;
        } else {
            stream.write_u8(0)?;
        }

        //    pub fee: u64,
        stream.write_u64::<BigEndian>(meta.fee)?;
       
        //    pub pre_balances: Vec<u64>,
        stream.write_u64::<BigEndian>(meta.pre_balances.len() as u64)?;
        for pb in &meta.pre_balances{
            stream.write_u64::<BigEndian>(*pb)?;
        }
        
        //    pub post_balances: Vec<u64>,
        stream.write_u64::<BigEndian>(meta.post_balances.len() as u64)?;
        for pb in &meta.post_balances{
            stream.write_u64::<BigEndian>(*pb)?;
        }


        //    pub compute_units_consumed: Option<u64>,
        match meta.compute_units_consumed{
            Some(x) => {
                stream.write_u64::<BigEndian>(x)?;
            },
            None => {
                stream.write_u64::<BigEndian>(0)?;
            },
        }
        

        //    SKIP: pub inner_instructions: Option<Vec<InnerInstructions>>,
        //    SKIP: pub log_messages: Option<Vec<String>>,

        //    SKIP: pub rewards: Option<Rewards>,
        //    SKIP: pub loaded_addresses: LoadedAddresses,
        //    SKIP: pub return_data: Option<TransactionReturnData>,
        
        

        Ok(())
    }
}





pub(crate) fn abort() -> ! {
    #[cfg(not(test))]
    {
        // standard error is usually redirected to a log file, cry for help on standard output as
        // well
        eprintln!("Validator process aborted. The validator log may contain further details");
        std::process::exit(1);
    }

    #[cfg(test)]
    panic!("process::exit(1) is intercepted for friendly test failure...");
}