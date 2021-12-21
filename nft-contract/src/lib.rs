use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{NFT_METADATA_SPEC, NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata};
use near_sdk::{AccountId, BorshStorageKey, env, near_bindgen, PanicOnDefault, Promise, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::serde::{Deserialize, Serialize};

use crate::web4::*;

mod web4;
const DEFAULT_ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAABmJLR0QA/wD/AP+gvaeTAAACAUlEQVRIidXVOWhVURAG4C8vxmAwcSmCCIIiWrjggmBhYyfYiQoKoiDaCGKrha2FWAiCQqxcUlmlsBEttBAUJNHGQlwCbhGekSDRkOTF4szBm+vbsMvAcA+z/LOec1no1NlC340zuIQqtmMz1uE6luMlZv8n+GmMYS74aImzfAynGoFUGsjuYgD9bSTSj5u40wDvHzostWOuxKOYCh6to6/iUBmsPIM1WItH4bShkNWysO+Mc6YZDOFa+PzARFYWS6pIQxRZvm6z5E68CR/YUfQrVrBe2o6c7UVpi0jlDwQP4V1U2osObMOTCLIYv/C9HGAQu/AW+8Ipg1/Ae6kdM/iMp9iDJQE6jY84Fv6DInqmL1iFyQDpC/lVPG/Qnt04F+cJLEJPYK0Wgkwr4ttTAnnVALys6yucV+ZDW3vbhDpaGRQDjMd3UmHNsLWJ/5bCeSJ8iQEzv0Uj6MI9aUgHQn5EWtmfJfClocv0EA+kizqdhcUteia9K1P4hL3SdvRK21KNzLqwE2elpRDBb0QVw9LTMc78HlawPwBz+ee1ntMcLkuvag52H7UMmqlWMOrGpmzUgmax0d9LOdzKr9lj9zv4Qx19FQfbSEgFt+sANPofZL6lTjvr9beG4ziJb20kNBa2J9RpTbNf5oj0BH+Vtulx2NekQfbjSiTzoo1EFij9AUQdkBPH3hCPAAAAAElFTkSuQmCC";
const NFT_CSS_SOURCE: &str = "/style";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NfTCheckers {
    owner_id: AccountId,
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval
}

#[near_bindgen]
impl NfTCheckers {
    #[init]
    pub fn new_with_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "NFT Checkers".to_string(),
                symbol: "NCHK".to_string(),
                icon: Some(DEFAULT_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            owner_id: owner_id.clone(),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "ERR_NO_ACCESS");
        self.tokens.internal_mint(token_id, receiver_id, Some(token_metadata))
    }

    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        let path = request.path.expect("Path expected");

        let token_id = get_token_id(&path).unwrap_or_default();

        if !token_id.is_empty() {
            if path.starts_with(NFT_CSS_SOURCE) {
                if let Some(token) = self.tokens.nft_token(token_id) {
                    let owner_id_css = token.owner_id.to_string().replace(".", "_").replace("-", "_");
                    let token_id_css = token.token_id.to_string().replace(".", "_").replace(" ", "_").replace("-", "_");
                    return Web4Response::css_response(
                        format!("div#board .piece.{}.{} {{
background-image: url('{}');
background-size: cover;
background-repeat: unset;
}}", owner_id_css, token_id_css, token.metadata.expect("ERR_MISSING_DATA").media.unwrap_or_default()));
                }
            }
        }

        Web4Response::webpage_template("NEAR NFT CHECKERS".to_string())
    }
}

fn get_token_id(token_id: &TokenId) -> Option<String> {
    let start = token_id.rfind('/').unwrap_or(0);
    let end = token_id.rfind('.').unwrap_or(0);

    if start * end != 0 && start + 1 < end {
        Some(token_id[start+1..end].to_string())
    }
    else {
        None
    }
}

near_contract_standards::impl_non_fungible_token_core!(NfTCheckers, tokens);
near_contract_standards::impl_non_fungible_token_approval!(NfTCheckers, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(NfTCheckers, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for NfTCheckers {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
