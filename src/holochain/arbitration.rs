//! Community arbitration system for Amazon Rose Forest

use hdk::prelude::*;
use crate::holochain::entries::{
    ArbitrationCase, ArbitrationStatus, ArbitrationVote, ArbitrationState
};
use crate::holochain::utils::{sys_time, create_path, timestamp_tag};
use std::collections::HashMap;

/// Input for conflict arbitration
#[derive(Serialize, Deserialize, Debug)]
pub struct ConflictInput {
    pub content_hash: String,
    pub description: String,
    pub key_insight: String,
    pub resolution: String,
}

/// Output from conflict arbitration
#[derive(Serialize, Deserialize, Debug)]
pub enum ArbitrationResult {
    Accept,
    Neutral,
    Reject,
}

/// Create a new arbitration case
#[hdk_extern]
pub fn create_arbitration_case(input: ConflictInput) -> ExternResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = sys_time()?;
    
    let case = ArbitrationCase {
        id: id.clone(),
        content_hash: input.content_hash,
        reporter: agent_info()?.agent_latest_pubkey,
        status: ArbitrationStatus::Open,
        votes: Vec::new(),
        resolution: None,
        created_at: now,
        updated_at: now,
    };
    
    // Create entry
    let case_hash = create_entry(&case)?;
    
    // Add to index
    let path = create_path("arbitration_cases", vec!["open"])?;
    create_link(path.path_entry_hash()?, case_hash, timestamp_tag())?;
    
    // Add to user's cases
    let my_cases_path = create_path("my_arbitration_cases", 
        vec![&agent_info()?.agent_latest_pubkey.to_string()])?;
    create_link(my_cases_path.path_entry_hash()?, case_hash, timestamp_tag())?;
    
    // Notify the network about the new case
    let notification = ArbitrationNotification {
        case_id: id.clone(),
        action: "new_case".to_string(),
        timestamp: now,
    };
    
    let signal = ExternIO::encode(notification)?;
    remote_signal(signal, RemoteSignal::All)?;
    
    Ok(id)
}

/// Vote on an arbitration case
#[hdk_extern]
pub fn vote_on_arbitration(input: VoteInput) -> ExternResult<()> {
    // Get the case
    let case_hash = get_arbitration_case_hash(&input.case_id)?;
    let mut case: ArbitrationCase = get_entry(case_hash.clone())?
        .ok_or(wasm_error!(WasmErrorInner::Guest("Case not found".to_string())))?
        .try_into()
        .map_err(|e: SerializedBytesError| wasm_error!(WasmErrorInner::Serialize(e)))?;
    
    // Validate vote
    if case.status != ArbitrationStatus::Open && case.status != ArbitrationStatus::UnderReview {
        return Err(wasm_error!(
            WasmErrorInner::Guest("Cannot vote on a closed case".to_string())
        ));
    }
    
    let agent_pubkey = agent_info()?.agent_latest_pubkey;
    
    // Check if agent has already voted
    if case.votes.iter().any(|v| v.voter == agent_pubkey) {
        return Err(wasm_error!(
            WasmErrorInner::Guest("Agent has already voted on this case".to_string())
        ));
    }
    
    // Add vote
    let now = sys_time()?;
    let vote = ArbitrationVote {
        voter: agent_pubkey,
        vote: input.vote,
        weight: input.weight.unwrap_or(1.0),
        justification: input.justification,
        timestamp: now,
    };
    
    case.votes.push(vote);
    case.updated_at = now;
    
    // Update case status if needed
    update_case_status(&mut case)?;
    
    // Update entry
    update_entry(case_hash.clone(), &case)?;
    
    // Notify about vote
    let notification = ArbitrationNotification {
        case_id: input.case_id.clone(),
        action: "new_vote".to_string(),
        timestamp: now,
    };
    
    let signal = ExternIO::encode(notification)?;
    remote_signal(signal, RemoteSignal::All)?;
    
    Ok(())
}

/// Trinary arbitration logic as described in the spec
#[hdk_extern]
pub fn arbitrate_conflict(input: ConflictInput) -> ExternResult<ArbitrationResult> {
    // Evaluate conflict using multiple perspectives
    let community_assessment = gather_community_input(&input)?;
    let semantic_analysis = analyze_semantic_intent(&input)?;
    let historical_context = retrieve_participant_history(&input)?;
    
    // Trinary logic allows nuanced outcomes
    match evaluate_conflict(&community_assessment, &semantic_analysis, &historical_context) {
        ArbitrationState::Resolve => {
            // Create learning opportunity from conflict
            create_knowledge_from_resolution(&input)?;
            Ok(ArbitrationResult::Accept)
        },
        ArbitrationState::Review => {
            // Escalate to broader community
            request_expanded_review(&input)?;
            Ok(ArbitrationResult::Neutral)
        },
        ArbitrationState::Reject => {
            // Apply minimum necessary intervention
            apply_restorative_measures(&input)?;
            Ok(ArbitrationResult::Reject)
        }
    }
}

/// Input for voting on arbitration cases
#[derive(Serialize, Deserialize, Debug)]
pub struct VoteInput {
    pub case_id: String,
    pub vote: ArbitrationState,
    pub justification: String,
    pub weight: Option<f32>,
}

/// Notification about arbitration activity
#[derive(Serialize, Deserialize, Debug)]
pub struct ArbitrationNotification {
    pub case_id: String,
    pub action: String,
    pub timestamp: u64,
}

/// Get the entry hash for an arbitration case
fn get_arbitration_case_hash(case_id: &str) -> ExternResult<EntryHash> {
    // Search in open cases
    let open_path = create_path("arbitration_cases", vec!["open"])?;
    let open_links = get_links(open_path.path_entry_hash()?, None)?;
    
    // Search in closed cases
    let closed_path = create_path("arbitration_cases", vec!["closed"])?;
    let closed_links = get_links(closed_path.path_entry_hash()?, None)?;
    
    let all_links = [open_links, closed_links].concat();
    
    for link in all_links {
        let entry: ArbitrationCase = get_entry(link.target.clone())?
            .ok_or(wasm_error!(WasmErrorInner::Guest("Case not found".to_string())))?
            .try_into()
            .map_err(|e: SerializedBytesError| wasm_error!(WasmErrorInner::Serialize(e)))?;
            
        if entry.id == case_id {
            return Ok(link.target);
        }
    }
    
    Err(wasm_error!(
        WasmErrorInner::Guest(format!("Case with ID {} not found", case_id))
    ))
}

/// Update the status of an arbitration case based on votes
fn update_case_status(case: &mut ArbitrationCase) -> ExternResult<()> {
    if case.votes.is_empty() {
        return Ok(());
    }
    
    // Count votes
    let mut resolve_votes = 0;
    let mut review_votes = 0;
    let mut reject_votes = 0;
    
    for vote in &case.votes {
        match vote.vote {
            ArbitrationState::Resolve => resolve_votes += 1,
            ArbitrationState::Review => review_votes += 1,
            ArbitrationState::Reject => reject_votes += 1,
        }
    }
    
    // Update status based on votes
    // This is a simple majority rule, but could be more sophisticated
    let total_votes = resolve_votes + review_votes + reject_votes;
    
    if total_votes >= 5 {  // Minimum threshold for decision
        if resolve_votes > total_votes / 2 {
            case.status = ArbitrationStatus::Resolved;
            case.resolution = Some("Community resolved this case positively".to_string());
        } else if reject_votes > total_votes / 2 {
            case.status = ArbitrationStatus::Rejected;
            case.resolution = Some("Community rejected this case".to_string());
        } else if review_votes > total_votes / 3 {
            case.status = ArbitrationStatus::UnderReview;
        }
    }
    
    Ok(())
}

/// Gather input from the community about a conflict
fn gather_community_input(input: &ConflictInput) -> ExternResult<Vec<CommunityInput>> {
    // This would query for community input in a real implementation
    // For now, this is a stub
    Ok(Vec::new())
}

/// Analyze the semantic intent of content
fn analyze_semantic_intent(input: &ConflictInput) -> ExternResult<SemanticAnalysis> {
    // This would use an embedding model in a real implementation
    // For now, this is a stub
    Ok(SemanticAnalysis {
        intent: "neutral".to_string(),
        confidence: 0.8,
    })
}

/// Retrieve historical context for participants
fn retrieve_participant_history(input: &ConflictInput) -> ExternResult<ParticipantHistory> {
    // This would query historical data in a real implementation
    // For now, this is a stub
    Ok(ParticipantHistory {
        participant_count: 1,
        average_contributions: 10,
    })
}

/// Evaluate a conflict using multiple perspectives
fn evaluate_conflict(
    community_assessment: &[CommunityInput],
    semantic_analysis: &SemanticAnalysis, 
    historical_context: &ParticipantHistory
) -> ArbitrationState {
    // This is a simplified implementation
    // In a real-world scenario, this would be more sophisticated
    
    if semantic_analysis.intent == "harmful" && semantic_analysis.confidence > 0.9 {
        return ArbitrationState::Reject;
    }
    
    if semantic_analysis.intent == "positive" && semantic_analysis.confidence > 0.7 {
        return ArbitrationState::Resolve;
    }
    
    // Default to review for anything ambiguous
    ArbitrationState::Review
}

/// Create knowledge from resolution
fn create_knowledge_from_resolution(input: &ConflictInput) -> ExternResult<()> {
    use crate::holochain::entries::KnowledgeContribution;
    use crate::holochain::entries::Metadata;
    use crate::holochain::utils::{hash_content, generate_embedding};
    
    let knowledge = KnowledgeContribution {
        content_hash: hash_content(&input.resolution),
        embedding: generate_embedding(&input.resolution),
        metadata: Metadata {
            tags: vec!["conflict_resolution".to_string(), "community_wisdom".to_string()],
            lesson_learned: input.key_insight.clone(),
        },
        timestamp: sys_time()?,
    };
    
    create_entry(&knowledge)?;
    
    // Index the knowledge
    let path = create_path("knowledge", vec!["conflict_resolution"])?;
    let knowledge_hash = hash_entry(&knowledge)?;
    create_link(path.path_entry_hash()?, knowledge_hash, timestamp_tag())?;
    
    Ok(())
}

/// Request expanded review for a case
fn request_expanded_review(input: &ConflictInput) -> ExternResult<()> {
    // This would notify additional reviewers in a real implementation
    // For now, this is a stub
    Ok(())
}

/// Apply restorative measures
fn apply_restorative_measures(input: &ConflictInput) -> ExternResult<()> {
    // This would apply appropriate interventions in a real implementation
    // For now, this is a stub
    Ok(())
}

/// Community input struct
#[derive(Debug)]
struct CommunityInput {
    agent: AgentPubKey,
    assessment: ArbitrationState,
    weight: f32,
}

/// Semantic analysis result
#[derive(Debug)]
struct SemanticAnalysis {
    intent: String,
    confidence: f32,
}

/// Participant history
#[derive(Debug)]
struct ParticipantHistory {
    participant_count: usize,
    average_contributions: usize,
}