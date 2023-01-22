



use crate::server::{
    FirehosePipe, start_firehose,
};

/// Main entry for the Firehose plugin
use {
    crate::{
        accounts_selector::AccountsSelector,
        transaction_selector::TransactionSelector,
    },
    log::*,
    serde_derive::{Deserialize, Serialize},
    serde_json,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
        ReplicaTransactionInfoVersions, Result, SlotStatus,
    },
    thiserror::Error,
    std::{
        fs::File, io::Read,
        str,
        sync::{
            mpsc::{self, Sender, Receiver},
            Mutex, 
            Arc,
        },
    },
    
};

#[derive(Default)]
pub struct GeyserPluginFirehose {
    directory: Option<String>,
    accounts_selector: Option<AccountsSelector>,
    transaction_selector: Option<TransactionSelector>,
    batch_starting_slot: Option<u64>,
    pipe: Option<FirehosePipe>,
    closer: Option<Arc<Mutex<Sender<i32>>>>,
}

impl std::fmt::Debug for GeyserPluginFirehose {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}


/// The Configuration for the Firehose plugin
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeyserPluginFirehoseConfig {
    pub grpc_listen_url: Option<String>,
}




#[derive(Error, Debug)]
pub enum GeyserPluginFirehoseError {

    #[error("Error preparing data store schema. Error message: ({msg})")]
    ConfigurationError { msg: String },

    #[error("Replica account V0.0.1 not supported anymore")]
    ReplicaAccountV001NotSupported,

    #[error("Failed to acquire lock.")]
    LockNotAcquired,
}


impl GeyserPlugin for GeyserPluginFirehose {
    fn name(&self) -> &'static str {
        "GeyserPluginFirehose"
    }

    fn on_load(&mut self, config_file: &str) -> Result<()> {
        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );
        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let result: serde_json::Value = serde_json::from_str(&contents).unwrap();
        self.accounts_selector = Some(Self::create_accounts_selector_from_config(&result));
        self.transaction_selector = Some(Self::create_transaction_selector_from_config(&result));

        let config: GeyserPluginFirehoseConfig =
            serde_json::from_str(&contents).map_err(|err| {
                GeyserPluginError::ConfigFileReadError {
                    msg: format!(
                        "The config file is not in the JSON format expected: {:?}",
                        err
                    ),
                }
            })?;
        info!("directory: {:?}",config.grpc_listen_url);

        if config.grpc_listen_url.is_none(){
            return Err(GeyserPluginError::Custom(GeyserPluginFirehoseError::ConfigurationError { msg: "no directory".to_string() }.into()))
        }

        let router = start_firehose()?;
        //let r_addr: SocketAddr = config.grpc_listen_url.unwrap().parse()?;
        let (tx, rx): (Sender<i32>, Receiver<i32>)=mpsc::channel();
        let _r_on=router.serve_with_shutdown(
            config.grpc_listen_url.unwrap().parse().unwrap(),
            async { let _ = rx.recv(); },
        );
        self.closer=Some(Arc::new(Mutex::new(tx)));
     
        Ok(())
    }

    fn on_unload(&mut self) {
        info!("Unloading plugin: {:?}", self.name());
        
        let can_close = self.closer.is_none();
        if !can_close {
            return
        }
        let tx = self.closer.as_ref().unwrap().clone();
        let r_tx = tx.lock();
        if !r_tx.is_err(){
            return
        }
        let _result = r_tx.unwrap().send(0);
        
        return
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> Result<()> {
        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> Result<()> {
        Ok(())
    }

    fn notify_transaction(
        &mut self,
        transaction_info: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> Result<()> {

        Ok(())
    }

    fn notify_block_metadata(&mut self, block_info: ReplicaBlockInfoVersions) -> Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        return true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        return false
    }
}

impl GeyserPluginFirehose {

    fn create_accounts_selector_from_config(config: &serde_json::Value) -> AccountsSelector {
        let accounts_selector = &config["accounts_selector"];

        if accounts_selector.is_null() {
            AccountsSelector::default()
        } else {
            let accounts = &accounts_selector["accounts"];
            let accounts: Vec<String> = if accounts.is_array() {
                accounts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            let owners = &accounts_selector["owners"];
            let owners: Vec<String> = if owners.is_array() {
                owners
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            AccountsSelector::new(&accounts, &owners)
        }
    }

    fn create_transaction_selector_from_config(config: &serde_json::Value) -> TransactionSelector {
        let transaction_selector = &config["transaction_selector"];

        if transaction_selector.is_null() {
            TransactionSelector::default()
        } else {
            let accounts = &transaction_selector["mentions"];
            let accounts: Vec<String> = if accounts.is_array() {
                accounts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            TransactionSelector::new(&accounts)
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}


#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns the GeyserPluginPostgres pointer as trait GeyserPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = GeyserPluginFirehose::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

#[cfg(test)]
pub(crate) mod tests {
    use {super::*, serde_json};

    #[test]
    fn test_accounts_selector_from_config() {
        let config = "{\"accounts_selector\" : { \
           \"owners\" : [\"9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin\"] \
        }}";

        let config: serde_json::Value = serde_json::from_str(config).unwrap();
        GeyserPluginFirehose::create_accounts_selector_from_config(&config);
    }
}
