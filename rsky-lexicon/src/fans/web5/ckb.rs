use crate::com::atproto::repo::Blob;
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreCreateAccountInput {
    pub handle: String,
    pub did: String,
    pub signing_key: Option<String>,
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreCreateAccountOutput {
    pub did: String,
    pub rev: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    pub version: u8,
    pub un_sign_bytes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountInput {
    pub handle: String,
    pub signing_key: String,
    pub password: Option<String>,
    pub root: SignedRoot,
    pub ckb_addr: String,
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountOutput {
    pub handle: String,
    pub did: String,
    #[serde(rename = "didDoc", skip_serializing_if = "Option::is_none")]
    pub did_doc: Option<Value>,
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
}

/// Pre apply writes output
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "fans.web5.ckb.createAccount#signedRoot")]
pub struct SignedRoot {
    pub did: String,
    pub rev: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    pub version: u8,
    pub signed_bytes: String,
}

/// Pre apply a batch transaction of repository creates, updates, and deletes.
/// Requires auth, implemented by PDS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PreDirectWritesInput {
    /// The handle or DID of the repo (aka, current account).
    pub repo: String,
    /// Can be set to 'false' to skip Lexicon schema validation of record data, for all operations.
    pub validate: Option<bool>,
    /// The Record Key.
    pub writes: Vec<PreDirectWritesInputRefWrite>,
    /// Compare and swap with the previous commit by CID.
    #[serde(rename = "swapCommit", skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
}

/// Direct apply a batch transaction of repository creates, updates, and deletes.
/// Requires auth, implemented by PDS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectWritesInput {
    /// The handle or DID of the repo (aka, current account).
    pub repo: String,
    /// Can be set to 'false' to skip Lexicon schema validation of record data, for all operations.
    pub validate: Option<bool>,
    /// The Record Key.
    pub writes: Vec<DirectWritesInputRefWrite>,
    /// Compare and swap with the previous commit by CID.
    #[serde(rename = "swapCommit", skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
    /// Signing bytes on PreDirectWritesInput return
    pub signing_key: String,
    pub ckb_addr: Option<String>,
    pub root: SignedRoot,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum PreDirectWritesInputRefWrite {
    #[serde(rename = "fans.web5.ckb.preDirectWrites#create")]
    Create(RefWriteCreate),
    #[serde(rename = "fans.web5.ckb.preDirectWrites#update")]
    Update(RefWriteUpdate),
    #[serde(rename = "fans.web5.ckb.preDirectWrites#delete")]
    Delete(RefWriteDelete),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum DirectWritesInputRefWrite {
    #[serde(rename = "fans.web5.ckb.directWrites#create")]
    Create(RefWriteCreate),
    #[serde(rename = "fans.web5.ckb.directWrites#update")]
    Update(RefWriteUpdate),
    #[serde(rename = "fans.web5.ckb.directWrites#delete")]
    Delete(RefWriteDelete),
}

/// Operation which creates a new record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefWriteCreate {
    pub collection: String,
    pub rkey: Option<String>,
    pub value: Value,
}

/// Operation which updates an existing record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefWriteUpdate {
    pub collection: String,
    pub rkey: String,
    pub value: Value,
}

/// Operation which deletes an existing record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefWriteDelete {
    pub collection: String,
    pub rkey: String,
}

/// Pre apply writes output
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreDirectWritesOutput {
    pub did: String,
    pub rev: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    pub version: u8,
    pub un_sign_bytes: String,
}

/// Precreate an authentication session.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreIndexActionInput {
    /// Handle or other identifier supported by the server for the authenticating user.
    pub did: String,
    pub ckb_addr: Option<String>,
    pub index: PreIndexActionInputRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum PreIndexActionInputRef {
    #[serde(rename = "fans.web5.ckb.preIndexAction#createSession")]
    CreateSessionIndex(RefCreateSessionIndex),
    #[serde(rename = "fans.web5.ckb.preIndexAction#deleteAccount")]
    DeleteAccountIndex(RefDeleteAccountIndex),
}

impl PreIndexActionInputRef {
    pub fn statement(&self) -> String {
        let domain = std::env::var("PDS_HOSTNAME").unwrap_or("web5.bbs.fans".into());
        match self {
            PreIndexActionInputRef::CreateSessionIndex(_) => {
                format!("Sign this message to authenticate with login on pds: {domain}.")
            }
            PreIndexActionInputRef::DeleteAccountIndex(_) => {
                format!("Sign this message to authenticate with delete account on pds: {domain}.")
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct RefCreateSessionIndex {}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct RefDeleteAccountIndex {}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreIndexActionOutput {
    /// Handle or other identifier supported by the server for the authenticating user.
    pub did: String,
    pub handle: String,
    pub message: String,
}

/// Create an authentication session.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexActionInput {
    /// Handle or other identifier supported by the server for the authenticating user.
    pub did: String,
    pub message: String,
    pub signing_key: String,
    pub signed_bytes: String,
    pub ckb_addr: Option<String>,
    pub index: IndexActionInputRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum IndexActionInputRef {
    #[serde(rename = "fans.web5.ckb.indexAction#createSession")]
    CreateSessionIndex(RefCreateSessionIndex),
    #[serde(rename = "fans.web5.ckb.indexAction#deleteAccount")]
    DeleteAccountIndex(RefDeleteAccountIndex),
}

impl IndexActionInputRef {
    pub fn statement(&self) -> String {
        let domain = std::env::var("PDS_HOSTNAME").unwrap_or("web5.bbs.fans".into());
        match self {
            IndexActionInputRef::CreateSessionIndex(_) => {
                format!("Sign this message to authenticate with login on pds: {domain}.")
            }
            IndexActionInputRef::DeleteAccountIndex(_) => {
                format!("Sign this message to authenticate with delete account on pds: {domain}.")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexActionOutput {
    pub result: IndexActionOutputRefResult,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum IndexActionOutputRefResult {
    #[serde(rename = "fans.web5.ckb.indexAction#createSessionResult")]
    CreateSessionResult(RefCreateSessionResult),
    #[serde(rename = "fans.web5.ckb.indexAction#deleteAccountResult")]
    DeleteAccountResult(RefDeleteAccountResult),
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct RefCreateSessionResult {
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
    #[serde(rename = "didDoc", skip_serializing_if = "Option::is_none")]
    pub did_doc: Option<Value>,
    pub email: Option<String>,
    #[serde(rename = "emailConfirmed", skip_serializing_if = "Option::is_none")]
    pub email_confirmed: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct RefDeleteAccountResult {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectWritesOutput {
    pub commit: Option<CommitMeta>,
    pub results: Option<Vec<DirectWritesOutputRefWrite>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum DirectWritesOutputRefWrite {
    #[serde(rename = "fans.web5.ckb.directWrites#createResult")]
    Create(RefWriteCreateResult),
    #[serde(rename = "fans.web5.ckb.directWrites#updateResult")]
    Update(RefWriteUpdateResult),
    #[serde(rename = "fans.web5.ckb.directWrites#deleteResult")]
    Delete(RefWriteDeleteResult),
}

/// Operation which creates a new record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefWriteCreateResult {
    pub uri: String,
    pub cid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_status: Option<String>,
}

/// Operation which updates an existing record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefWriteUpdateResult {
    pub uri: String,
    pub cid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_status: Option<String>,
}

/// Operation which deletes an existing record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefWriteDeleteResult {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename = "com.atproto.repo.defs#commitMeta")]
pub struct CommitMeta {
    pub cid: String,
    pub rev: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobOutput {
    pub blob_server: String,
    pub blob: Blob,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexQueryInput {
    /// Handle or other identifier supported by the server for the authenticating user.
    pub index: IndexQueryInputRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum IndexQueryInputRef {
    #[serde(rename = "fans.web5.ckb.indexQuery#firstItem")]
    First(FirstIndex),
    #[serde(rename = "fans.web5.ckb.indexQuery#secondItem")]
    Second(SecondIndex),
    #[serde(rename = "fans.web5.ckb.indexQuery#thirdItem")]
    Third(ThirdIndex),
    #[serde(rename = "fans.web5.ckb.indexQuery#fourthItem")]
    Fourth(FourthIndex),
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct FirstIndex {}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct SecondIndex {}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct ThirdIndex {
    pub did: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct FourthIndex {
    pub did: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexQueryOutput {
    /// Handle or other identifier supported by the server for the authenticating user.
    pub result: IndexQueryOutputRef,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum IndexQueryOutputRef {
    #[serde(rename = "fans.web5.ckb.indexQuery#firstItemResult")]
    First(FirstResult),
    #[serde(rename = "fans.web5.ckb.indexQuery#secondItemResult")]
    Second(SecondResult),
    #[serde(rename = "fans.web5.ckb.indexQuery#thirdItemResult")]
    Third(ThirdResult),
    #[serde(rename = "fans.web5.ckb.indexQuery#fourthItemResult")]
    Fourth(FourthResult),
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct FirstResult {
    pub result: usize,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct SecondResult {
    pub result: usize,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct ThirdResult {
    pub result: usize,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct FourthResult {
    pub result: String,
}
