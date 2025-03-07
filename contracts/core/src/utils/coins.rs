use std::{collections::BTreeMap, fmt};

use cosmwasm_std::{Coin, Uint128, OverflowError};

// denom => amount
#[derive(Debug, PartialEq, Eq)]
pub struct Coins(BTreeMap<String, Uint128>);

// UNSAFE: because we don't check for duplicate denoms or zero amounts
// only use this for trusted coin vecs, such as MessageInfo::funds
impl From<Vec<Coin>> for Coins {
    fn from(coin_vec: Vec<Coin>) -> Self {
        Self(coin_vec
            .into_iter()
            .map(|coin| (coin.denom, coin.amount))
            .collect())
    }
}

// NOTE: the output vec is guaranteed to be ordered alphabetically ascendingly
// by the denoms
impl From<Coins> for Vec<Coin> {
    fn from(coins: Coins) -> Self {
        coins
            .0
            .into_iter()
            .map(|(denom, amount)| Coin {
                denom,
                amount,
            })
            .collect()
    }
}

impl fmt::Display for Coins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "[]");
        }

        let s = self
            .0
            .iter()
            .map(|(denom, amount)| format!("{amount}{denom}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{s}")
    }
}

impl Coins {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add(&mut self, new_coin: Coin) -> Result<(), OverflowError> {
        let amount = self.0.entry(new_coin.denom).or_insert_with(Uint128::zero);
        *amount = amount.checked_add(new_coin.amount)?;
        Ok(())
    }
}
