pub use hdk_extensions::hdi;
pub use hdk_extensions::hdk;
pub use hdk_extensions::hdi_extensions;
pub use hdk_extensions;

use hdi::prelude::*;
use hdk::prelude::debug;
use hdi_extensions::{
    ScopedTypeConnector, scoped_type_connector,
};
pub use hdi_extensions::{
    // Macros
    valid, invalid,
};
pub use test_types::{
    PostEntry,
};



#[hdk_entry_types]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_type]
    Post(PostEntry),
}

scoped_type_connector!(
    EntryTypesUnit::Post,
    EntryTypes::Post( PostEntry )
);


#[hdk_link_types]
pub enum LinkTypes {
    Generic,
}


#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::StoreRecord(op_record) => match op_record {
            // OpRecord::CreateEntry { app_entry, action } =>
            //         create_entry::validation( app_entry, action ),
            OpRecord::UpdateEntry { app_entry, action, original_action_hash, original_entry_hash } =>
                update_entry_validation( app_entry, action, original_action_hash, original_entry_hash ),
            // OpRecord::DeleteEntry { original_action_hash, original_entry_hash, action: delete },
            // OpRecord::CreateLink { base_address, target_address, tag, link_type, action: update_link },
            // OpRecord::DeleteLink { original_action_hash, base_address, action: delete_link },
            // OpRecord::CreateAgent { agent, action: create },
            // OpRecord::UpdateAgent { original_key, new_key, original_action_hash, action: update },
            // OpRecord::CreateCapClaim { action: create },
            // OpRecord::CreateCapGrant { action: create },
            // OpRecord::CreatePrivateEntry { app_entry_type, action: create },
            // OpRecord::UpdatePrivateEntry { original_action_hash, original_entry_hash, app_entry_type, action: update },
            // OpRecord::UpdateCapClaim { original_action_hash, original_entry_hash, action: update },
            // OpRecord::UpdateCapGrant { original_action_hash, original_entry_hash, action: update },
            // OpRecord::Dna { dna_hash, action: dna },
            // OpRecord::OpenChain { previous_dna_hash, action: open_chain },
            // OpRecord::CloseChain { new_dna_hash, action: close_chain },
            // OpRecord::AgentValidationPkg { membrane_proof, action: agent_validation_pkg },
            // OpRecord::InitZomesComplete { action: init_zomes_complete },
            _ => valid!(),
        },
        // FlatOp::StoreEntry(op_entry),
        // FlatOp::RegisterAgentActivity(op_activity),
        // FlatOp::RegisterCreateLink { base_address, target_address, tag, link_type, action: create_link },
        // FlatOp::RegisterDeleteLink { original_action, base_address, target_address, tag, link_type, action: delete_link },
        // FlatOp::RegisterUpdate(op_update),
        // FlatOp::RegisterDelete(op_delete),
        _ => valid!(),
    }
}

pub fn update_entry_validation(
    app_entry: EntryTypes,
    _update: Update,
    _original_action_hash: ActionHash,
    _original_entry_hash: EntryHash
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
        EntryTypes::Post(post) => {
            debug!("Checking EntryTypes::Post({:#?})", post );
            valid!()
        },
    }
}
