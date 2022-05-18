use cosmwasm_std::{
    entry_point, to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,Binary,
    StdResult, Uint128,CosmosMsg,WasmMsg,BankMsg,QueryRequest,WasmQuery,Delegation,DistributionMsg, StakingMsg,StakingQuery,FullDelegation};

use terra_cosmwasm::{TerraMsgWrapper,create_swap_msg};    
use cw2::set_contract_version;
use cw20::{ Cw20ExecuteMsg, Cw20QueryMsg};
use crate::oracle::QueryMsg as OracleQueryMsg;

use crate::error::{ContractError};
use crate::msg::{ ExecuteMsg, InstantiateMsg};
use crate::state::{State,DelegationResponse, CONFIG,VALIDATOR};


const CONTRACT_NAME: &str = "SWAP_CONTRACT";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
   
    let state = State {
        owner : msg.owner,
        oracle_address:msg.oracle_address,
        token_address:msg.token_address,
    };
    CONFIG.save(deps.storage,&state)?;
    VALIDATOR.save(deps.storage, &msg.validator)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    match msg {
    ExecuteMsg::BuyLemons2{} =>execute_buy_lemons(deps,env,info),
    ExecuteMsg::WithdrawAmount2 { amount }=>execute_withdraw_amount(deps,env,info,amount),
    ExecuteMsg::StartUndelegation2 { amount } =>execute_set_undelegation(deps,env,info,amount)
    }
}

fn execute_buy_lemons(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    let  state = CONFIG.load(deps.storage)?;
    let validator = VALIDATOR.load(deps.storage)?;
    
    let deposit_amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uluna".to_string())
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    let lemon_price:Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.oracle_address.to_string(),
        msg: to_binary(&OracleQueryMsg::GetPrice { })?,
    }))?;

    let buyable_token_amount = deposit_amount/lemon_price;

    let availabe_token_amount:Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.oracle_address.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance { address: env.contract.address.to_string() })?,
    }))?;

    if availabe_token_amount<buyable_token_amount{
        return Err(ContractError::NotEnoughTokens {})
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: buyable_token_amount,
            })?,
        })
    )
    .add_message(CosmosMsg::Staking(
        StakingMsg::Delegate {
            validator:validator , 
            amount: Coin {
                 denom: "uluna".to_string(), 
                 amount: deposit_amount } }
    ))
)
}

fn execute_withdraw_amount(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    amount:Uint128
) -> Result<Response<TerraMsgWrapper>, ContractError> {
 let  state = CONFIG.load(deps.storage)?;
 let validator = VALIDATOR.load(deps.storage)?;
 
 if state.owner !=info.sender.to_string(){
     return Err(ContractError::Unauthorized {  })
 }

 let delegation_response :DelegationResponse = deps.querier.query(&QueryRequest::Staking(
     StakingQuery::Delegation {
          delegator: env.contract.address.to_string(), 
          validator: validator.clone() }))?;

 let delegation = delegation_response.delegation.unwrap();
 let reward = delegation.accumulated_rewards;
 let mut swap_messages:Vec<CosmosMsg<TerraMsgWrapper>> = vec![];
 for i in 0..reward.len(){
     swap_messages.push(create_swap_msg(
       reward[i].clone(),
       "uluna".to_string()
     ))
 }
 
 Ok(Response::new()
        .add_message(CosmosMsg::Distribution
            (DistributionMsg::SetWithdrawAddress{ 
                address: env.contract.address.to_string() 
            }))
        .add_message(CosmosMsg::Distribution(
            DistributionMsg::WithdrawDelegatorReward { 
                validator: validator }))
         .add_messages(swap_messages)    
         .add_message(
         CosmosMsg::Bank(BankMsg::Send {
            to_address: state.owner.to_string(),
            amount:vec![Coin{
                    denom:"uluna".to_string(),
                    amount:amount
                }]
        })
        )
        )

}




fn execute_set_undelegation(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    amount:Uint128
) -> Result<Response<TerraMsgWrapper>, ContractError> {
 let  state = CONFIG.load(deps.storage)?;
 let validator = VALIDATOR.load(deps.storage)?;
 
 if state.owner !=info.sender.to_string(){
     return Err(ContractError::Unauthorized {  })
 }

 
 Ok(Response::new().add_message(
     CosmosMsg::Staking(StakingMsg::Undelegate 
        { validator: validator, 
            amount: Coin{
                denom:"uluna".to_string(),
                amount:amount
            } })
        )
    )

}
