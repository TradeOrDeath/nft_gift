use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::collections::HashSet;

// Define the NFT structure
#[derive(BorshSerialize, BorshDeserialize)]
pub struct NFT {
    pub owner_id: AccountId,
    pub token_id: u64,
    pub image_url: String,
}

// Define the contract state
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NFTContract {
    pub nfts: Vector<NFT>,
    pub owner_id: AccountId,
    pub next_public_token_id: u64,
    pub allowed_claimers: HashSet<AccountId>,
}

#[near_bindgen]
impl NFTContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            nfts: Vector::new(b"n".to_vec()),
            owner_id,
            next_public_token_id: 1,
            allowed_claimers: HashSet::new(),
        }
    }

    pub fn mint(&mut self, token_id: u64, image_url: String, allowed_claimers: Option<Vec<AccountId>>) -> bool {
    let caller = env::signer_account_id();
    if caller != self.owner_id {
        env::panic(b"Only the owner can mint NFTs.");
    }
    
        if let Some(claimers_list) = allowed_claimers {
            // If a list of allowed claimers is provided, add them to the HashSet
            for claimer in claimers_list {
                self.allowed_claimers.insert(claimer);
            }
        } else {
            // If no list is provided, mint 100 NFTs that can be claimed by anyone
            for i in 0..100 {
                self.nfts.push(&NFT {
                    owner_id: self.owner_id.clone(),
                    token_id: self.next_public_token_id,
                    image_url: image_url.clone(), 
                });
                self.next_public_token_id += 1;
            }
        }
        true
    }
    
    pub fn transfer(&mut self, receiver_id: AccountId, token_id: u64) -> bool {
        let caller = env::signer_account_id();
        let mut nft = match self.nfts.get(token_id) {
            Some(nft) => nft,
            None => return false, // NFT with the given token_id doesn't exist
        };
        if nft.owner_id != caller {
            return false; // Caller is not the owner of the NFT
        }

        nft.owner_id = receiver_id;
        true
    }
    
    pub fn claim(&mut self) -> bool {
        let caller = env::signer_account_id();
        if self.allowed_claimers.is_empty() || self.allowed_claimers.contains(&caller) {
            self.nfts.push(&NFT {
                owner_id: caller,
                token_id: self.next_public_token_id,
                image_url: "".to_string(), 
            });
            self.next_public_token_id += 1;
            true
        } else {
            false
        }
    }
    
}
// ... (code for the contract)

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{MockedBlockchain, testing_env};

    // Helper function to initialize the contract
    fn init_contract(owner_id: AccountId) -> NFTContract {
        NFTContract::new(owner_id)
    }

    #[test]
    fn test_mint() {
        let context = VMContextBuilder::new()
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build(); // Build the VMContext here
        testing_env!(context);

        // Initialize the contract with the owner account ID
        let mut contract = init_contract(accounts(0).to_string());

        // The owner can mint NFTs with allowed claimers list
        let allowed_claimers = vec![accounts(1).to_string(), accounts(2).to_string()];
        assert!(contract.mint(1, "http://example.com/nft1".to_string(), Some(allowed_claimers.clone())));

        // The minted NFT should be added to the contract's nfts vector
        let nft = contract.nfts.get(0).unwrap();
        assert_eq!(nft.owner_id, accounts(0).to_string());
        assert_eq!(nft.token_id, 1);
        assert_eq!(nft.image_url, "http://example.com/nft1");

        // Non-owner should not be able to mint NFTs
        let context_non_owner = VMContextBuilder::new()
            .current_account_id(accounts(1))
            .predecessor_account_id(accounts(1))
            .build(); // Build the VMContext for non-owner account
        testing_env!(context_non_owner);
        assert_eq!(contract.mint(2, "http://example.com/nft2".to_string(), None), false);
    }

    #[test]
    fn test_claim() {
        let context = VMContextBuilder::new()
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build(); // Build the VMContext here
        testing_env!(context);

        // Initialize the contract with the owner account ID
        let mut contract = init_contract(accounts(0).to_string());

        // Non-allowed claimer should not be able to claim NFTs
        let context_non_claimer = VMContextBuilder::new()
            .current_account_id(accounts(3))
            .predecessor_account_id(accounts(3))
            .build(); // Build the VMContext for non-allowed claimer account
        testing_env!(context_non_claimer);
        assert_eq!(contract.claim(), false);

        // Allowed claimer should be able to claim NFTs
        let allowed_claimers = vec![accounts(3).to_string()];
        contract.mint(1, "http://example.com/nft1".to_string(), Some(allowed_claimers));
        assert!(contract.claim());

        // The claimed NFT should be added to the contract's nfts vector
        let nft = contract.nfts.get(0).unwrap();
        assert_eq!(nft.owner_id, accounts(3).to_string());
        assert_eq!(nft.token_id, 2); // Next token ID after the minted ones
        assert_eq!(nft.image_url, "");
    }
}
