use std::fmt;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PaperMetadata {
    pub title: Option<String>,
    pub reviewers: HashMap<AccountId,Reviewdata>,
    pub vote_yes: u64,
    pub vote_rev: u64,
    pub vote_no: u64,
    pub status: Status,
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Vote {
    Yes,
    Review,
    No,
    NotVoted,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Status {
    Published,
    InReview,
    Unpublished,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Approval {
    Approved,
    AwaitApprov,
    NotApproved,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Pay {
    Payed,
    NotPayed,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Reviewdata {
    pub accept: Approval,
    pub vote: Vote,
    pub payedrev: Pay,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title:          Option<String>,      // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description:    Option<String>,      // free-form description
    pub media:          Option<String>,      // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash:     Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies:         Option<u64>,         // number of copies of this set of metadata in existence when token was minted.
    pub issued_at:      Option<u64>,         // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at:     Option<u64>,         // When token expires, Unix epoch in milliseconds
    pub starts_at:      Option<u64>,         // When token starts being valid, Unix epoch in milliseconds
    pub updated_at:     Option<u64>,         // When token was last updated, Unix epoch in milliseconds
    pub extra:          Option<String>,      // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference:      Option<String>,      // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    pub owner_id: AccountId,                           //owner of the token
    pub approved_account_ids: HashMap<AccountId, u64>, //list of approved account IDs that have access to transfer the token. 
                                                       //This maps an account ID to an approval ID
    pub next_approval_id: u64,                         //the next approval ID to give out. 
    pub royalty: HashMap<AccountId, u32>,              //keep track of the royalty percentages for the token in a hash map
}

// Interface to capture data about an event
//
// Arguments:
// * `standard`: name of standard e.g. nep171
// * `version`: e.g. 1.0.0
// * `event`: associate event data
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    #[serde(flatten)]
    pub event: EventLogVariant, // `flatten` to not have "event": {<EventLogVariant>} in the JSON,
                                // just have the contents of {<EventLogVariant>}.
}

impl fmt::Display for EventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}


// Enum that represents the data type of the EventLog.
// The enum can either be an NftMint or an NftTransfer.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    NftMint(Vec<NftMintLog>),
    NftTransfer(Vec<NftTransferLog>),
}


// An event log to capture token minting
//
// Arguments
// * `owner_id`: "account.near"
// * `token_ids`: ["1", "abc"]
// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftMintLog {
    pub owner_id: String,
    pub token_ids: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// An event log to capture token transfer
///
/// Arguments
/// * `authorized_id`: approved account to transfer
/// * `old_owner_id`: "owner.near"
/// * `new_owner_id`: "receiver.near"
/// * `token_ids`: ["1", "12345abc"]
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftTransferLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<String>,

    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}
