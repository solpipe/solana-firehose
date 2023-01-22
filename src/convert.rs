use solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoVersions;

use crate::{
    pipe,
    basic,
};

pub(crate) fn convert_pubkey(pubkey: &[u8])->basic::Pubkey{
    let mut data = Vec::new();
    for i in 0..pubkey.len()-1{
        data.push(pubkey[i]);
    }
    
    return basic::Pubkey{
        data,
    }
}

pub(crate) fn convert_signature(signature: Option<&solana_sdk::signature::Signature>)->Option<basic::Signature>{
    
    if let Some(s) = signature {
        let x = s.as_ref();
        let mut data = Vec::new();
        for i in 0..x.len()-1{
            data.push(x[i]);
        }
        
        return Some(basic::Signature{
            data,
        })
    }else{
        None
    }
    
}

pub(crate) fn convert_data(input: &[u8])->Vec<u8>{
    let mut ans = Vec::new();
    let iter = input.iter();
    for item in iter{
        ans.push(*item);
    }
    return ans;
}

pub(crate) fn convert_account(account: ReplicaAccountInfoVersions)->Option<pipe::Account>{
    
    match account{
        ReplicaAccountInfoVersions::V0_0_1(a) => {
            return Some(pipe::Account{
                pubkey: Some(convert_pubkey(a.pubkey)),
                lamports:a.lamports,
                owner: Some(convert_pubkey(a.owner)),
                executable: a.executable,
                rent_epoch: a.rent_epoch,
                data:convert_data(a.data),
                write_version: a.write_version,
                txn_signature: None,
            })

        },
        ReplicaAccountInfoVersions::V0_0_2(a) => {
            return Some(pipe::Account{
                pubkey: Some(convert_pubkey(a.pubkey)),
                lamports:a.lamports,
                owner: Some(convert_pubkey(a.owner)),
                executable: a.executable,
                rent_epoch: a.rent_epoch,
                data:convert_data(a.data),
                write_version: a.write_version,
                txn_signature: convert_signature(a.txn_signature),
            })
        },
        _=>{return None},
    }
}



