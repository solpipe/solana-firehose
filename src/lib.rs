
pub mod basic {
    tonic::include_proto!("basic");
}

pub mod pipe{
    tonic::include_proto!("pipe");
}


pub mod accounts_selector;
pub mod firehose;
//pub mod utility;
//pub mod broadcast;
pub mod geyser_plugin_firehose;
pub mod inline_spl_token;
pub mod inline_spl_token_2022;
pub mod transaction_selector;
