pub use hdi_extensions::hdi;
pub use hdi_extensions::holo_hash;
pub use hdk;
pub use hdi_extensions;

use core::convert::{ TryFrom, TryInto };
use hdi_extensions::{
    summon_action,
    summon_entry,
};
use hdk::prelude::{
    get, get_details, agent_info,
    debug, wasm_error,
    Serialize, Deserialize,
    ExternResult, WasmError, WasmErrorInner, GetOptions,
    Record, Action, Details, RecordDetails, SignedHashed,
    LinkTypeFilter, LinkTypeFilterExt, LinkTag,
};
use holo_hash::{
    AgentPubKey, ActionHash, AnyDhtHash, AnyLinkableHash,
    AnyDhtHashPrimitive, AnyLinkableHashPrimitive,
};
use thiserror::Error;
use hdi_extensions::*;



//
// General Structs
//
/// A distinct state within the context of a life-cycle
///
/// A [`MorphAddr`] and its entry content represent a chain in the entity's life-cycle.
///
/// ##### Example: Basic Usage
/// ```
/// # use hdk::prelude::{
///     ActionHash, TryFrom,
/// };
/// # use hdk_extensions::{
///     Entity, MorphAddr,
/// };
///
/// #[derive(Clone)]
/// struct Content {
///     pub message: String,
/// }
///
/// let identity_addr = "uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7";
/// let revision_addr = "uhCkknDrZjzEgzf8iIQ6aEzbqEYrYBBg1pv_iTNUGAFJovhxOJqu0";
///
/// Entity::<Content>(
///     MorphAddr(
///         ActionHash::try_from(identity_addr).unwrap(),
///         ActionHash::try_from(revision_addr).unwrap(),
///     ),
///     Content {
///         message: String::from("Hello world"),
///     }
/// );
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Entity<T>(
    /// The Metamorphic address relevant to `T` (the content)
    pub MorphAddr,
    /// The content that belong's to the [`MorphAddr`]'s revision address
    pub T,
)
where
    T: Clone;

impl<T> Entity<T>
where
    T: Clone,
{
    /// See [`MorphAddr::is_origin`]
    pub fn identity(&self) -> &ActionHash {
        self.0.identity()
    }

    /// See [`MorphAddr::is_origin`]
    pub fn revision(&self) -> &ActionHash {
        self.0.revision()
    }

    /// See [`MorphAddr::is_origin`]
    pub fn is_origin(&self) -> bool {
        self.0.is_origin()
    }
}

/// An address representing a precise phase in an entities life-cycle (short for: Metamorphic
/// Address)
///
/// Together the pair of identity/revision addresses act as coordinates that can be used to
/// determine the entity's identity (identity addr) and a phase in its life-cycle (revision addr).
///
/// ##### Example: Basic Usage
/// ```
/// # use hdk::prelude::{
///     ActionHash, TryFrom,
/// };
/// # use hdk_extensions::{
///     Entity, MorphAddr,
/// };
///
/// let identity_addr = "uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7";
/// let revision_addr = "uhCkknDrZjzEgzf8iIQ6aEzbqEYrYBBg1pv_iTNUGAFJovhxOJqu0";
///
/// MorphAddr(
///     ActionHash::try_from(identity_addr).unwrap(),
///     ActionHash::try_from(revision_addr).unwrap(),
/// );
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MorphAddr(
    /// The create action of the entities life-cycle
    pub ActionHash,
    /// Any entry creation action in the entity's life-cycle
    pub ActionHash,
);

impl MorphAddr {
    /// A reference to the tuple's index 0
    pub fn identity(&self) -> &ActionHash {
        &self.0
    }

    /// A reference to the tuple's index 1
    pub fn revision(&self) -> &ActionHash {
        &self.1
    }

    /// This is an origin metamorphic address if the identity and revision are the same
    pub fn is_origin(&self) -> bool {
        self.0 == self.1
    }
}



//
// Custom Errors
//
#[derive(Debug, Error)]
pub enum HdkExtError<'a> {
    #[error("Record not found @ address {0}")]
    RecordNotFound(&'a AnyDhtHash),
    #[error("No entry in record ({0})")]
    RecordHasNoEntry(&'a ActionHash),
    #[error("Expected an action hash, not an entry hash: {0}")]
    ExpectedRecordNotEntry(&'a ActionHash),
}

impl<'a> From<HdkExtError<'a>> for WasmError {
    fn from(error: HdkExtError) -> Self {
        wasm_error!(WasmErrorInner::Guest( format!("{}", error ) ))
    }
}



//
// Agent
//
/// Get this Agent's initial pubkey from zome info
pub fn agent_id() -> ExternResult<AgentPubKey> {
    Ok( agent_info()?.agent_initial_pubkey )
}



//
// Get Helpers
//
/// Get a [`Record`] or return a "not found" error
///
/// The difference between this `must_get` and `hdk`'s `get` is that this one replaces a `None` response
/// with [`HdkExtError::RecordNotFound`] so that an ok result will always be a [`Record`].
///
/// **NOTE:** Not to be confused with the `hdi`'s meaning of 'must'.  This 'must' will not retrieve
/// deleted records.
pub fn must_get<T>(addr: &T) -> ExternResult<Record>
where
    T: Clone + std::fmt::Debug,
    AnyDhtHash: From<T>,
{
    Ok(
        get( addr.to_owned(), GetOptions::latest() )?
            .ok_or(HdkExtError::RecordNotFound(&addr.to_owned().into()))?
    )
}


/// Get the [`RecordDetails`] for a given [`ActionHash`]
///
/// This method provides a more deterministic result by unwrapping the [`get_details`] result.
pub fn must_get_record_details(action: &ActionHash) -> ExternResult<RecordDetails> {
    let details = get_details( action.to_owned(), GetOptions::latest() )?
        .ok_or(HdkExtError::RecordNotFound(&action.to_owned().into()))?;

    match details {
        Details::Record(record_details) => Ok( record_details ),
        Details::Entry(_) => Err(HdkExtError::ExpectedRecordNotEntry(action))?,
    }
}


/// Check if a DHT address can be fetched
pub fn exists<T>(addr: &T) -> ExternResult<bool>
where
    T: Clone + std::fmt::Debug,
    AnyDhtHash: From<T>,
{
    debug!("Checking if address {:?} exists", addr );
    Ok(
        match AnyDhtHash::from(addr.to_owned()).into_primitive() {
            AnyDhtHashPrimitive::Action(addr) => summon_action( &addr ).is_ok(),
            AnyDhtHashPrimitive::Entry(addr) => summon_entry( &addr ).is_ok(),
        }
    )
}


/// Check if a DHT address can be fetched and is not deleted
pub fn available<T>(addr: &T) -> ExternResult<bool>
where
    T: Clone + std::fmt::Debug,
    AnyDhtHash: From<T>,
{
    debug!("Checking if address {:?} is available", addr );
    Ok( get( addr.to_owned(), GetOptions::latest() )?.is_some() )
}



//
// Tracing Actions
//
/// Resolve an [`AnyLinkableHash`] into an [`ActionHash`]
///
/// If the linkable's primitive is a
/// - `Action` - the action hash is simply returned
/// - `Entry` - the action hash is pulled from the result of a `get`
/// - `External` - results in an error
pub fn resolve_action_addr<T>(addr: &T) -> ExternResult<ActionHash>
where
    T: Into<AnyLinkableHash> + Clone,
{
    let addr : AnyLinkableHash = addr.to_owned().into();
    match addr.into_primitive() {
        AnyLinkableHashPrimitive::Entry(entry_hash) => {
            Ok(
                must_get( &entry_hash )?.action_address().to_owned()
            )
        },
        AnyLinkableHashPrimitive::Action(action_hash) => Ok( action_hash ),
        AnyLinkableHashPrimitive::External(external_hash) => Err(guest_error!(
            format!("External hash ({}) will not have a corresponding action", external_hash )
        )),
    }
}


/// Collect the chain of evolutions forward
///
/// When there are multiple updates the lowest action's timestamp is selected.
///
/// The first item of the returned [`Vec`] will always be the given [`ActionHash`]
pub fn follow_evolutions(action_address: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let mut evolutions = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
        let details = must_get_record_details( &addr )?;
        let maybe_next_update = details.updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        next_addr = match maybe_next_update {
            Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
            None => None,
        };

        evolutions.push( addr );
    }

    Ok( evolutions )
}


/// Collect the chain of evolutions forward filtering updates
pub fn follow_evolutions_selector<F>(
    action_address: &ActionHash,
    selector: F
) -> ExternResult<Vec<ActionHash>>
where
    F: Fn(Vec<SignedHashed<Action>>) -> ExternResult<Option<ActionHash>>,
{
    let mut evolutions = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
        let details = must_get_record_details( &addr )?;
        next_addr = selector( details.updates )?;

        evolutions.push( addr );
    }

    Ok( evolutions )
}


/// Collect the chain of evolutions forward filtering authorized updates
pub fn follow_evolutions_using_authorities(
    action_address: &ActionHash,
    authors: &Vec<AgentPubKey>
) -> ExternResult<Vec<ActionHash>> {
    let evolutions = follow_evolutions_selector( action_address, |updates| {
        let updates_count = updates.len();
        let valid_updates : Vec<SignedHashed<Action>> = updates
            .into_iter()
            .filter(|sa| {
                debug!(
                    "Checking authorities for author '{}': {:?}",
                    sa.action().author(),
                    authors
                );
                authors.contains( sa.action().author() )
            })
            .collect();

        debug!(
            "Filtered {}/{} updates",
            updates_count - valid_updates.len(),
            updates_count
        );
        let maybe_next_update = valid_updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        Ok(
            match maybe_next_update {
                Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
                None => None,
            }
        )
    })?;

    Ok( evolutions )
}


/// Collect the chain of evolutions forward filtering authorized updates with exceptions
pub fn follow_evolutions_using_authorities_with_exceptions(
    action_address: &ActionHash,
    authors: &Vec<AgentPubKey>,
    exceptions: &Vec<ActionHash>
) -> ExternResult<Vec<ActionHash>> {
    let evolutions = follow_evolutions_selector( action_address, |updates| {
        let updates_count = updates.len();
        let valid_updates : Vec<SignedHashed<Action>> = updates
            .into_iter()
            .filter(|sa| {
                debug!(
                    "Checking authorities for author '{}' or an action exception '{}'",
                    sa.action().author(),
                    sa.action_address()
                );
                authors.contains( sa.action().author() ) || exceptions.contains( sa.action_address() )
            })
            .collect();

        debug!(
            "Filtered {}/{} updates",
            updates_count - valid_updates.len(),
            updates_count
        );
        let maybe_next_update = valid_updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        Ok(
            match maybe_next_update {
                Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
                None => None,
            }
        )
    })?;

    Ok( evolutions )
}


/// Indicates the update filtering pattern when following content evolution
#[derive(Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum EvolutionFilteringStrategy {
    /// Variant used for [`follow_evolutions`]
    Unfiltered,
    /// Variant used for [`follow_evolutions_using_authorities`]
    AuthoritiesFilter(Vec<AgentPubKey>),
    /// Variant used for [`follow_evolutions_using_authorities_with_exceptions`]
    AuthoritiesExceptionsFilter(Vec<AgentPubKey>, Vec<ActionHash>),
    /// Not used yet
    ExceptionsFilter(Vec<ActionHash>),
}

impl Default for EvolutionFilteringStrategy {
    fn default() -> Self {
        EvolutionFilteringStrategy::Unfiltered
    }
}

impl<'de> serde::Deserialize<'de> for EvolutionFilteringStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let buffer : FollowEvolutionsInputBuffer = Deserialize::deserialize(deserializer)?;

        Ok( buffer.into() )
    }
}

/// A deserializing buffer for [`EvolutionFilteringStrategy`]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FollowEvolutionsInputBuffer {
    pub authors: Option<Vec<AgentPubKey>>,
    pub exceptions: Option<Vec<ActionHash>>,
}

impl From<FollowEvolutionsInputBuffer> for EvolutionFilteringStrategy {
    fn from(buffer: FollowEvolutionsInputBuffer) -> Self {
        match (buffer.authors, buffer.exceptions) {
            (None, None) => Self::Unfiltered,
            (Some(authors), None) => Self::AuthoritiesFilter(authors),
            (None, Some(exceptions)) => Self::ExceptionsFilter(exceptions),
            (Some(authors), Some(exceptions)) => Self::AuthoritiesExceptionsFilter(authors, exceptions),
        }
    }
}


//
// Standard Inputs
//
/// Input required for calling the [`follow_evolutions`] method
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetEntityInput {
    pub id: ActionHash,
    #[serde(default)]
    pub follow_strategy: EvolutionFilteringStrategy,
}

/// Input required for calling the [`hdk::prelude::update_entry`] method
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateEntryInput<T> {
    pub base: ActionHash,
    pub entry: T,
}

/// A simpler deserializable buffer for [`GetLinksInput`]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetLinksInputBuffer {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub link_type: String,
    pub tag: Option<String>,
}

/// Input required for calling the [`hdk::prelude::get_links`] method
#[derive(Clone, Serialize, Debug)]
pub struct GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub link_type_filter: LinkTypeFilter,
    pub tag: Option<LinkTag>,
    pub link_type: Option<T>,
}

impl<T> TryFrom<GetLinksInputBuffer> for GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    type Error = WasmError;

    fn try_from(buffer: GetLinksInputBuffer) -> Result<Self, Self::Error> {
        let (link_type, link_type_filter) = match buffer.link_type.as_str() {
            ".." => ( None, (..).try_into_filter()? ),
            name => {
                let link_type = T::try_from( name.to_string() )?;
                ( Some(link_type.clone()), link_type.try_into_filter()? )
            },
        };

        Ok(Self {
            base: buffer.base,
            target: buffer.target,
            tag: buffer.tag.map(|text| text.into_bytes().into() ),
            link_type,
            link_type_filter,
        })
    }
}

impl<'de,T> serde::Deserialize<'de> for GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let buffer : GetLinksInputBuffer = Deserialize::deserialize(deserializer)?;
        let error_msg = format!("Buffer could not be converted: {:#?}", buffer );

        Ok(
            buffer.try_into()
                .or(Err(serde::de::Error::custom(error_msg)))?
        )
    }
}
