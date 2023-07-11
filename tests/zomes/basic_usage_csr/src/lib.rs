use hdk::prelude::*;
use hdk_extensions::{
    must_get,
    // Entity, MorphAddr,

    // HDI Extensions
    ScopedTypeConnector,
    // Inputs
    GetEntityInput,
    UpdateEntryInput,
};
use basic_usage::{
    PostEntry,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'{}' init", zome_info()?.name );
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    Ok( agent_info()? )
}


#[hdk_extern]
pub fn create_post(post: PostEntry) -> ExternResult<ActionHash> {
    debug!("Creating new post entry: {:#?}", post );
    let action_hash = create_entry( post.to_input() )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_post(input: GetEntityInput) -> ExternResult<PostEntry> {
    debug!("Get latest post entry: {:#?}", input );
    let record = must_get( &input.id )?;

    Ok( PostEntry::try_from_record( &record )? )
}


#[hdk_extern]
pub fn update_post(input: UpdateEntryInput<PostEntry>) -> ExternResult<ActionHash> {
    debug!("Update post action: {}", input.base );
    // let prev_post : PostEntry = must_get( &input.base )?.try_into();
    let action_hash = update_entry( input.base, input.entry.to_input() )?;

    Ok( action_hash )
}
