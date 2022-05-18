

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{ Uint128};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner : String,
    pub oracle_address : String,
    pub token_address:String,
    pub validator: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    BuyLemons2{},
    WithdrawAmount2{amount:Uint128},
    StartUndelegation2{amount:Uint128}
}



