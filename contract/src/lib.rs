use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, PanicOnDefault, Promise, CryptoHash, Balance,
//    BorshStorageKey, PromiseOrValue, 
};
use std::collections::HashMap;
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap,
//    UnorderedSet
};
use near_sdk::json_types::{Base64VecU8,
//    U128
};

//use near_contract_standards::non_fungible_token::metadata::{TokenMetadata};//,NFT_METADATA_SPEC,NFTContractMetadata};
//use near_sdk::json_types::ValidAccountId;

pub use crate::traits::*;
mod traits; 

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

pub const NFT_METADATA_SPEC: &str = "nft-1.0.0"; // This spec can be treated like a version of the standard.
pub const NFT_STANDARD_NAME: &str = "nep171";    // This is the name of the NFT standard we're using

//near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner: AccountId,
    pub paperid: LookupMap<TokenId, Token>,
    pub tokenmetadata: UnorderedMap<TokenId, TokenMetadata>,
    pub papersmetadata: UnorderedMap<TokenId, PaperMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    PaperMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new(metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let papers = Self{
            owner: env::predecessor_account_id(),
            paperid:        LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            tokenmetadata:  UnorderedMap::new(StorageKey::TokenMetadataById.try_to_vec().unwrap()),
            papersmetadata: UnorderedMap::new(StorageKey::PaperMetadataById.try_to_vec().unwrap()),
            metadata:       LazyOption::new(
                                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                                Some(&metadata)),
        };
        papers
    }

    #[init]
    pub fn new_standard() -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "Bitcoin: A Peer-to-Peer Electronic Cash System".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some("https://bitcoin.org/bitcoin.pdf".to_string()),
            base_uri: None,
            reference: None,
            reference_hash: None,
        };

        let papers = Self{
            owner: env::predecessor_account_id(),
            paperid:        LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            tokenmetadata:  UnorderedMap::new(StorageKey::TokenMetadataById.try_to_vec().unwrap()),
            papersmetadata: UnorderedMap::new(StorageKey::PaperMetadataById.try_to_vec().unwrap()),
            metadata:       LazyOption::new(
                                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                                Some(&metadata)),
        };
        papers
    }

    #[payable]
    pub fn submit(&mut self, token_id: &TokenId,
        title: String, author: Vec<String>, accrev: Vec<AccountId>){
        assert!(env::attached_deposit()==ONE_NEAR*10,"Should deposit 10 Near.");
        assert!(accrev.len()==3,"It should be 3 reviewers!");
        Promise::new(env::current_account_id()).transfer(env::attached_deposit());
        
        let mut rev = HashMap::new();
        for account_id in accrev {
            let revdata = Reviewdata{accept: Approval::AwaitApprov, vote: Vote::NotVoted, payedrev: Pay::NotPayed};
            assert!(env::signer_account_id() != account_id.clone(),"Signer Cannot be Reviewer");
            rev.insert(account_id.clone(),revdata);
        }        

        let ppermtdt = PaperMetadata {
            title:          title,
            author:         author,
            reviewers:      rev,
            vote_yes:       0,
            vote_rev:       0,
            vote_no:        0,
            status:         Status::Unpublished,
        };
        self.papersmetadata.insert(&token_id,&ppermtdt);
    }

    pub fn stataccept(&mut self,token_id: &TokenId,approv: Approval){
        let account_id = env::predecessor_account_id();
        assert!(self.papersmetadata.get(&token_id).unwrap().reviewers.contains_key(&account_id));
        assert!(approv != Approval::AwaitApprov);


//        let reviewer = Reviewdata{accept: Approval::Approved, vote: Vote::NotVoted, payedrev: Pay::NotPayed};

        let mut a = self.papersmetadata.get(&token_id).unwrap();


        if approv==Approval::Approved{
            a.reviewers.get_mut(&account_id).unwrap().accept = Approval::Approved;
        }else{
            a.reviewers.remove(&account_id);
        }
        self.papersmetadata.insert(&token_id,&a);
    }

    pub fn addreviewer(&mut self,token_id: &TokenId, accrev: AccountId){
        assert!(env::signer_account_id() != accrev.clone(),"Signer Cannot be Reviewer");
        assert!(self.papersmetadata.get(&token_id).unwrap().reviewers.len()<3,"Already Maximum Number of Reviewers");

        let reviewer = Reviewdata{accept: Approval::Approved, vote: Vote::NotVoted, payedrev: Pay::NotPayed};
        self.papersmetadata.get(&token_id).unwrap().reviewers.insert(accrev.clone(),reviewer);
    }

    pub fn voting(&mut self,token_id: &TokenId,vote: Vote) {
        assert!(
            self.papersmetadata.get(&token_id).unwrap().reviewers.contains_key(&env::predecessor_account_id()),
            "Not a reviewer!"
        );
        let papmeta = self.papersmetadata.get(&token_id).unwrap();
        let review = papmeta.reviewers.get(&env::predecessor_account_id()).unwrap();
        assert!(review.vote == Vote::NotVoted,"Already Reviewed!");

        let mut a = self.papersmetadata.get(&token_id).unwrap();

        match vote{
            Vote::Yes =>    a.vote_yes += 1,
            Vote::Review => a.vote_rev += 1,
            Vote::No =>     a.vote_no  += 1,
            _ => (),
        }

        a.reviewers.get_mut(&env::predecessor_account_id()).unwrap().vote = vote;
        self.papersmetadata.insert(&token_id,&a);
    }

    #[payable]
    pub fn payreviewer(&mut self,token_id: &TokenId) {
        assert_eq!(&env::predecessor_account_id(),&env::current_account_id(),"Not Owner!");


        let mut a = self.papersmetadata.get(&token_id).unwrap();
        for (account_id,revdata) in self.papersmetadata.get(&token_id).unwrap().reviewers.iter_mut() {

            match revdata.vote{
                Vote::NotVoted => continue,
                _ => (),
            };

            match revdata.payedrev{
                Pay::NotPayed => (),
                _ => continue,
            };
            

            Promise::new(account_id.clone()).transfer(ONE_NEAR);
            revdata.payedrev = Pay::Payed;
//            a.reviewers.get_mut(&account_id.to_string()).unwrap().payedrev = Pay::Payed;
            a.reviewers.get_mut(&account_id).unwrap().payedrev = Pay::Payed;
        }
        self.papersmetadata.insert(&token_id,&a);
    }

    pub fn publish(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ){
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Only the owner can mint the contract!");

        let a = self.papersmetadata.get(&token_id).unwrap();

        for (acc,revdata) in a.reviewers.iter() {
            assert!(revdata.payedrev==Pay::Payed,"{} not payed", acc.to_string());
            assert!(revdata.accept==Approval::Approved,"{} not approved", acc.to_string());
        }
        assert!(a.vote_yes==3,"Not all reviewers have accepted");
        self.mint(token_id.clone(),token_metadata,receiver_id,None);

        let mut a = self.papersmetadata.get(&token_id).unwrap();
        a.status = Status::Published;
        self.papersmetadata.insert(&token_id,&a);

    }


    fn refund_deposit(storage_used: u64) {
        let required_cost = env::storage_byte_cost() * Balance::from(storage_used); //get how much it would cost to store 
                                                                                    //the information
        let attached_deposit = env::attached_deposit(); //get the attached deposit

        assert!( //make sure that the attached deposit is greater than or equal to the required cost
            required_cost <= attached_deposit,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );

        let refund = attached_deposit - required_cost; //get the refund amount from the attached deposit - required cost

        
        if refund > 1 { //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
            Promise::new(env::predecessor_account_id()).transfer(refund); 
        }
    }


    fn mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        _perpetual_royalties: Option<HashMap<AccountId, u32>> //we add an optional parameter for perpetual royalties
    ){
        
        let initial_storage_usage = env::storage_usage(); //measure the initial storage being used on the contract

        let royalty = HashMap::new();
        let token = Token {
            owner_id: receiver_id,                    //set the owner ID equal to the receiver ID passed into the function
            approved_account_ids: Default::default(), //we set the approved account IDs to the default value (an empty map)
            next_approval_id: 0,                      //the next approval ID is set to 0
            royalty,                                  //the map of perpetual royalties for the token (The owner will get 100%
                                                      // - total perpetual royalties)
        };
        assert!(
            self.paperid.insert(&token_id, &token).is_none(),
            "Paper already published"
        );

        self.tokenmetadata.insert(&token_id, &metadata);

        let nft_mint_log: EventLog = EventLog {               // Construct the mint log as per the events standard.
            standard: NFT_STANDARD_NAME.to_string(),          // Standard name ("nep171").
            version: NFT_METADATA_SPEC.to_string(),           // Version of the standard ("nft-1.0.0").
            event: EventLogVariant::NftMint(vec![NftMintLog { // The data related with the event stored in a vector.
                owner_id: token.owner_id.to_string(),         // Owner of the token.
                token_ids: vec![token_id.to_string()],        // Vector of token IDs that were minted.
                memo: None,                                   // An optional memo to include.
            }]),
        };
        env::log_str(&nft_mint_log.to_string()); // Log the serialized json.

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage; //calculate the required storage which
                                                                                      //was the used - initial

        Contract::refund_deposit(required_storage_in_bytes); //refund any excess storage if the user attached too much.
                                                             //Panic if they didn't attach enough to cover the required.
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::{testing_env,
//        MockedBlockchain
    };
    use near_sdk::test_utils::{accounts, VMContextBuilder};

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }


    fn tokenmeta() -> NFTContractMetadata {
        NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "Bitcoin: A Peer-to-Peer Electronic Cash System".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some("https://bitcoin.org/bitcoin.pdf".to_string()),
            base_uri: None,
            reference: None,
            reference_hash: None,
        }
    }


    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Bitcoin: A Peer-to-Peer Electronic Cash System".into()),
            description: Some("Article".into()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_basics() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        
        let mtdt = tokenmeta();

        let title = "Bitcoin: A Peer-to-Peer Electronic Cash System";
        let author = vec!["Nakamoto, Satoshi".to_string()];
        let token_id = "0";
        let mut cnt = Contract::new(mtdt);

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR*10)
                            .predecessor_account_id(accounts(1))
                            .build());

        cnt.submit(&token_id.to_string(),title.to_string(),author,
            vec![accounts(2),accounts(3),accounts(4)]
        );


        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(2)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::Yes);

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(3)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::Yes);

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(4)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::Yes);

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR)
                            .predecessor_account_id(accounts(0))
                            .build());
        cnt.payreviewer(&token_id.to_string());

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR)
                            .predecessor_account_id(accounts(0))
                            .build());
        cnt.publish(token_id.to_string(), accounts(1), sample_token_metadata());

        let a = cnt.papersmetadata.get(&"0".to_string()).unwrap();
        
        assert!(a.status==Status::Published);
    }

    #[test]
    #[should_panic]
    fn not_approved() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        
        let mtdt = tokenmeta();

        let title = "Bitcoin: A Peer-to-Peer Electronic Cash System";
        let author = vec!["Nakamoto, Satoshi".to_string()];
        let token_id = "0";
        let mut cnt = Contract::new(mtdt);

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR*10)
                            .predecessor_account_id(accounts(1))
                            .build());

        cnt.submit(&token_id.to_string(),title.to_string(),author,
            vec![accounts(2),accounts(3),accounts(4)]
        );
                        

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(2)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::No);

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(3)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::Yes);

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(4)).build());
        cnt.stataccept(&token_id.to_string(),Approval::Approved);
        cnt.voting(&token_id.to_string(),Vote::Yes);

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR)
                            .predecessor_account_id(accounts(0))
                            .build());
        cnt.payreviewer(&token_id.to_string());

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR)
                            .predecessor_account_id(accounts(0))
                            .build());
        cnt.publish(token_id.to_string(), accounts(1), sample_token_metadata());
    }

    #[test]
    #[should_panic]
    fn vote_not_approved() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        
        let mtdt = tokenmeta();

        let title = "Bitcoin: A Peer-to-Peer Electronic Cash System";
        let author = vec!["Nakamoto, Satoshi".to_string()];
        let token_id = "0";
        let mut cnt = Contract::new(mtdt);

        testing_env!(context.storage_usage(env::storage_usage())
                            .attached_deposit(ONE_NEAR*10)
                            .predecessor_account_id(accounts(1))
                            .build());

        cnt.submit(&token_id.to_string(),title.to_string(),author,
            vec![accounts(2),accounts(3),accounts(4)]
        );
                        

        testing_env!(VMContextBuilder::new().predecessor_account_id(accounts(2)).build());
        cnt.stataccept(&token_id.to_string(),Approval::NotApproved);
        cnt.voting(&token_id.to_string(),Vote::Yes);
    }

    #[test]
    fn nep_format_vector() {
        let expected = r#"EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":"foundation.near","token_ids":["aurora","proximitylabs"]},{"owner_id":"user1.near","token_ids":["meme"]}]}"#;
        let log = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftMint(vec![
                NftMintLog {
                    owner_id: "foundation.near".to_owned(),
                    token_ids: vec!["aurora".to_string(), "proximitylabs".to_string()],
                    memo: None,
                },
                NftMintLog {
                    owner_id: "user1.near".to_owned(),
                    token_ids: vec!["meme".to_string()],
                    memo: None,
                },
            ]),
        };
        assert_eq!(expected, log.to_string());
    }

    #[test]
    fn nep_format_mint() {
        let expected = r#"EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":"foundation.near","token_ids":["aurora","proximitylabs"]}]}"#;
        let log = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: "foundation.near".to_owned(),
                token_ids: vec!["aurora".to_string(), "proximitylabs".to_string()],
                memo: None,
            }]),
        };
        assert_eq!(expected, log.to_string());
    }

}
