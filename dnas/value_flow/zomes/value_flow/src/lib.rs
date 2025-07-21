use hdk::prelude::*;

#[hdk_entry(id = "value_flow")]
#[derive(Clone)]
pub struct ValueFlow {
    pub from: AgentPubKey,
    pub to: AgentPubKey,
    pub utility: f32,
    pub governance_weight: f32,
    pub reputation_shift: Vec<f32>,
}

#[hdk_extern]
pub fn create_value_flow(value_flow: ValueFlow) -> ExternResult<EntryHash> {
    let entry_hash = create_entry(&value_flow)?;
    let from_path = Path::from(format!("value_flow.{}", value_flow.from));
    create_link(from_path.path_entry_hash()?, entry_hash.clone(), LinkTag::new(vec![]))?;
    let to_path = Path::from(format!("value_flow.{}", value_flow.to));
    create_link(to_path.path_entry_hash()?, entry_hash.clone(), LinkTag::new(vec![]))?;
    Ok(entry_hash)
}

#[hdk_extern]
pub fn get_value_flows_for_agent(agent: AgentPubKey) -> ExternResult<Vec<ValueFlow>> {
    let path = Path::from(format!("value_flow.{}", agent));
    let links = get_links(path.path_entry_hash()?, None)?;
    let value_flows: Vec<ValueFlow> = links
        .into_inner()
        .iter()
        .map(|link| {
            get(link.target.clone(), GetOptions::default())
                .and_then(|element| {
                    element
                        .ok_or(wasm_error!(WasmErrorInner::Guest("ValueFlow not found".to_string())))
                })
                .and_then(|element| {
                    element
                        .entry()
                        .to_app_option()?
                        .ok_or(wasm_error!(WasmErrorInner::Guest("ValueFlow not found".to_string())))
                })
        })
        .collect::<ExternResult<Vec<ValueFlow>>>()?;
    Ok(value_flows)
}

#[hdk_entry(id = "reputation_shift")]
#[derive(Clone)]
pub struct ReputationShift {
    pub agent: AgentPubKey,
    pub context: String,
    pub shift_vector: Vec<f32>,
    pub description: String,
    pub timestamp: u64,
}

#[hdk_extern]
pub fn create_reputation_shift(reputation_shift: ReputationShift) -> ExternResult<EntryHash> {
    let entry_hash = create_entry(&reputation_shift)?;
    let path = Path::from(format!("reputation_shift.{}", reputation_shift.agent));
    create_link(path.path_entry_hash()?, entry_hash.clone(), LinkTag::new(vec![]))?;
    Ok(entry_hash)
}

#[hdk_extern]
pub fn get_reputation_shifts_for_agent(
    agent: AgentPubKey,
) -> ExternResult<Vec<ReputationShift>> {
    let path = Path::from(format!("reputation_shift.{}", agent));
    let links = get_links(path.path_entry_hash()?, None)?;
    let reputation_shifts: Vec<ReputationShift> = links
        .into_inner()
        .iter()
        .map(|link| {
            get(link.target.clone(), GetOptions::default())
                .and_then(|element| {
                    element.ok_or(wasm_error!(WasmErrorInner::Guest(
                        "ReputationShift not found".to_string()
                    )))
                })
                .and_then(|element| {
                    element.entry().to_app_option()?.ok_or(wasm_error!(
                        WasmErrorInner::Guest("ReputationShift not found".to_string())
                    ))
                })
        })
        .collect::<ExternResult<Vec<ReputationShift>>>()?;
    Ok(reputation_shifts)
}

use ad4m_client::Client;
use anyhow::Result;
use serde_json::json;

#[hdk_extern]
pub fn publish_reputation_shift(reputation_shift: ReputationShift) -> ExternResult<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = Client::new("http://localhost:4000").await.unwrap();
        let expression = json!({
            "author": reputation_shift.agent.to_string(),
            "timestamp": reputation_shift.timestamp,
            "data": {
                "type": "reputation_shift",
                "context": reputation_shift.context,
                "shift_vector": reputation_shift.shift_vector,
                "description": reputation_shift.description,
            }
        });
        client
            .expression()
            .create(expression.to_string(), vec![])
            .await
            .unwrap();
    });
    Ok(())
}

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}
