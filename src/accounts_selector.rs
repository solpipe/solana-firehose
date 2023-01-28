use {log::*, std::collections::HashSet};

#[derive(Debug)]
pub(crate) struct AccountsSelector {
    pub _accounts: HashSet<Vec<u8>>,
    pub _owners: HashSet<Vec<u8>>,
    pub _select_all_accounts: bool,
}

impl AccountsSelector {
    pub fn default() -> Self {
        AccountsSelector {
            _accounts: HashSet::default(),
            _owners: HashSet::default(),
            _select_all_accounts: true,
        }
    }

    pub fn new(accounts: &[String], owners: &[String]) -> Self {
        info!(
            "Creating AccountsSelector from accounts: {:?}, owners: {:?}",
            accounts, owners
        );

        let select_all_accounts = accounts.iter().any(|key| key == "*");
        if select_all_accounts {
            return AccountsSelector {
                _accounts: HashSet::default(),
                _owners: HashSet::default(),
                _select_all_accounts: select_all_accounts,
            };
        }
        let accounts = accounts
            .iter()
            .map(|key| bs58::decode(key).into_vec().unwrap())
            .collect();
        let owners = owners
            .iter()
            .map(|key| bs58::decode(key).into_vec().unwrap())
            .collect();
        AccountsSelector {
            _accounts: accounts,
            _owners: owners,
            _select_all_accounts: select_all_accounts,
        }
    }

}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_create_accounts_selector() {
        AccountsSelector::new(
            &["9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string()],
            &[],
        );

        AccountsSelector::new(
            &[],
            &["9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string()],
        );
    }
}
