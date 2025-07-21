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

#[hdk_entry(id = "reputation")]
#[derive(Clone)]
pub struct Reputation {
    pub agent: AgentPubKey,
    pub reputation: Vec<f32>,
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

#[hdk_extern]
pub fn get_reputation(agent: AgentPubKey) -> ExternResult<Option<Reputation>> {
    let path = Path::from(format!("reputation.{}", agent));
    let links = get_links(path.path_entry_hash()?, None)?;
    if let Some(link) = links.into_inner().get(0) {
        let reputation: Reputation = get(link.target.clone(), GetOptions::default())?
            .ok_or(wasm_error!(WasmErrorInner::Guest("Reputation not found".to_string())))?
            .entry()
            .to_app_option()?
            .ok_or(wasm_error!(WasmErrorInner::Guest("Reputation not found".to_string())))?;
        Ok(Some(reputation))
    } else {
        Ok(None)
    }
}

#[hdk_extern]
pub fn update_reputation(value_flow: ValueFlow) -> ExternResult<()> {
    let from_reputation = get_reputation(value_flow.from.clone())?
        .unwrap_or(Reputation {
            agent: value_flow.from.clone(),
            reputation: vec![0.5; value_flow.reputation_shift.len()],
        });
    let to_reputation = get_reputation(value_flow.to.clone())?
        .unwrap_or(Reputation {
            agent: value_flow.to.clone(),
            reputation: vec![0.5; value_flow.reputation_shift.len()],
        });

    let mut new_from_reputation = from_reputation.reputation.clone();
    let mut new_to_reputation = to_reputation.reputation.clone();

    for (i, shift) in value_flow.reputation_shift.iter().enumerate() {
        new_from_reputation[i] -= shift;
        new_to_reputation[i] += shift;
    }

    let new_from_reputation = Reputation {
        agent: value_flow.from,
        reputation: new_from_reputation,
    };
    let new_to_reputation = Reputation {
        agent: value_flow.to,
        reputation: new_to_reputation,
    };

    let from_entry_hash = create_entry(&new_from_reputation)?;
    let to_entry_hash = create_entry(&new_to_reputation)?;

    let from_path = Path::from(format!("reputation.{}", new_from_reputation.agent));
    create_link(from_path.path_entry_hash()?, from_entry_hash, LinkTag::new(vec![]))?;
    let to_path = Path::from(format!("reputation.{}", new_to_reputation.agent));
    create_link(to_path.path_entry_hash()?, to_entry_hash, LinkTag::new(vec![]))?;

    Ok(())
}
