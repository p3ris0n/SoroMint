#![cfg(test)]
use super::*;
use proptest::prelude::*;
use soroban_sdk::{
    testutils::Address as _, testutils::Events, Address, Env, IntoVal, String, Val,
};

fn setup() -> (Env, Address, Address, SoroMintTokenClient<'static>) {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user = Address::generate(&e);
    let token_id = e.register(SoroMintToken, ());
    let client = SoroMintTokenClient::new(&e, &token_id);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&e, "SoroMint"),
        &String::from_str(&e, "SMT"),
    );

    (e, admin, user, client)
}

fn last_event_data(e: &Env) -> Val {
    let events = e.events().all();
    let last = events.last().expect("expected at least one event");
    last.2
}

fn find_event_by_action(e: &Env, action: Val) -> Option<Val> {
    e.events()
        .all()
        .iter()
        .rev()
        .find(|(_, topics, _)| {
            let topic_values: Vec<Val> = topics.clone();
            topic_values.len() == 2
                && topic_values.get(1).unwrap().get_payload() == action.get_payload()
        })
        .map(|(_, _, data)| data)
}

#[test]
fn test_initialize_and_mint() {
    let (_, _, user, client) = setup();
    client.mint(&user, &1000);
    assert_eq!(client.balance(&user), 1000);
    assert_eq!(client.decimals(), 7);
    assert_eq!(client.name(), String::from_str(&client.env, "SoroMint"));
    assert_eq!(client.symbol(), String::from_str(&client.env, "SMT"));
}

#[test]
fn test_initialize_emits_event() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let token_id = e.register(SoroMintToken, ());
    let client = SoroMintTokenClient::new(&e, &token_id);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&e, "SoroMint"),
        &String::from_str(&e, "SMT"),
    );

    let data: (Address, u32, String, String) = last_event_data(&e).into_val(&e);
    assert_eq!(data.0, admin);
    assert_eq!(data.1, 7);
    assert_eq!(data.2, String::from_str(&e, "SoroMint"));
    assert_eq!(data.3, String::from_str(&e, "SMT"));
}

#[test]
fn test_mint_and_burn() {
    let (_, _, user, client) = setup();

    client.mint(&user, &1000);
    assert_eq!(client.balance(&user), 1000);
    assert_eq!(client.supply(), 1000);

    client.burn(&user, &400);
    assert_eq!(client.balance(&user), 600);
    assert_eq!(client.supply(), 600);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_burn_insufficient_balance() {
    let (_, _, user, client) = setup();
    client.mint(&user, &100);
    client.burn(&user, &200);
}

#[test]
fn test_burn_emits_event_with_new_balance_and_supply() {
    let (e, admin, user, client) = setup();

    client.mint(&user, &900);
    client.burn(&user, &300);

    let data: (Address, Address, i128, i128, i128) = last_event_data(&e).into_val(&e);
    assert_eq!(data.0, admin);
    assert_eq!(data.1, user);
    assert_eq!(data.2, 300);
    assert_eq!(data.3, 600);
    assert_eq!(data.4, 600);
}

#[test]
fn test_transfer() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.transfer(&user1, &user2, &300);

    assert_eq!(client.balance(&user1), 700);
    assert_eq!(client.balance(&user2), 300);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_transfer_to_self_keeps_same_balance_and_supply() {
    let (_, _, user, client) = setup();

    client.mint(&user, &500);
    client.transfer(&user, &user, &200);

    assert_eq!(client.balance(&user), 500);
    assert_eq!(client.supply(), 500);
}

#[test]
fn test_transfer_emits_expected_event() {
    let (e, _, user, client) = setup();
    let recipient = Address::generate(&e);

    client.mint(&user, &900);
    client.transfer(&user, &recipient, &250);

    let data: (Address, Address, i128, i128, i128) = last_event_data(&e).into_val(&e);
    assert_eq!(data.0, user);
    assert_eq!(data.1, recipient);
    assert_eq!(data.2, 250);
    assert_eq!(data.3, 650);
    assert_eq!(data.4, 250);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_transfer_insufficient_balance() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &200);
}

#[test]
#[should_panic(expected = "transfer amount must be positive")]
fn test_transfer_zero_panics() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &0);
}

#[test]
#[should_panic(expected = "Contract is paused")]
fn test_transfer_fails_when_paused() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.pause();
    client.transfer(&user1, &user2, &300);
}

#[test]
fn test_transfer_succeeds_after_unpause() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.pause();
    client.unpause();
    client.transfer(&user1, &user2, &300);

    assert_eq!(client.balance(&user1), 700);
}

#[test]
fn test_approve_and_transfer_from() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.approve(&user1, &user2, &500, &1000);

    assert_eq!(client.allowance(&user1, &user2), 500);

    client.transfer_from(&user2, &user1, &user3, &200);

    assert_eq!(client.balance(&user1), 800);
    assert_eq!(client.balance(&user3), 200);
    assert_eq!(client.allowance(&user1, &user2), 300);
}

#[test]
fn test_approve_overwrites_and_clears_allowance() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);

    client.approve(&owner, &spender, &300, &1000);
    assert_eq!(client.allowance(&owner, &spender), 300);

    client.approve(&owner, &spender, &125, &1000);
    assert_eq!(client.allowance(&owner, &spender), 125);

    client.approve(&owner, &spender, &0, &1000);
    assert_eq!(client.allowance(&owner, &spender), 0);
}

#[test]
fn test_approve_emits_expected_event() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);

    client.approve(&owner, &spender, &220, &1000);

    let data: (Address, Address, i128) = last_event_data(&e).into_val(&e);
    assert_eq!(data.0, owner);
    assert_eq!(data.1, spender);
    assert_eq!(data.2, 220);
}

#[test]
#[should_panic(expected = "approval amount must be non-negative")]
fn test_approve_negative_panics() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);

    client.approve(&owner, &spender, &-1, &1000);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn test_transfer_from_insufficient_allowance() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.approve(&user1, &user2, &100, &1000);
    client.transfer_from(&user2, &user1, &user3, &200);
}

#[test]
fn test_transfer_from_emits_expected_event() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);
    let recipient = Address::generate(&e);

    client.mint(&owner, &800);
    client.approve(&owner, &spender, &500, &1000);
    client.transfer_from(&spender, &owner, &recipient, &300);

    let action: Val = symbol_short!("xfer_from").into_val(&e);
    let event = find_event_by_action(&e, action).expect("expected transfer_from event");
    let data: (Address, Address, Address, i128, i128, i128, i128) = event.into_val(&e);
    assert_eq!(data.0, spender);
    assert_eq!(data.1, owner);
    assert_eq!(data.2, recipient);
    assert_eq!(data.3, 300);
    assert_eq!(data.4, 200);
    assert_eq!(data.5, 500);
    assert_eq!(data.6, 300);
}

#[test]
fn test_transfer_from_to_self_consumes_allowance_without_changing_balance() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);

    client.mint(&owner, &500);
    client.approve(&owner, &spender, &200, &1000);
    client.transfer_from(&spender, &owner, &owner, &120);

    assert_eq!(client.balance(&owner), 500);
    assert_eq!(client.allowance(&owner, &spender), 80);
    assert_eq!(client.supply(), 500);
}

#[test]
fn test_transfer_from_exhausts_allowance_exactly() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);
    let recipient = Address::generate(&e);

    client.mint(&owner, &250);
    client.approve(&owner, &spender, &250, &1000);
    client.transfer_from(&spender, &owner, &recipient, &250);

    assert_eq!(client.balance(&owner), 0);
    assert_eq!(client.balance(&recipient), 250);
    assert_eq!(client.allowance(&owner, &spender), 0);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_transfer_from_more_than_balance_panics_even_with_allowance() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);
    let recipient = Address::generate(&e);

    client.mint(&owner, &100);
    client.approve(&owner, &spender, &300, &1000);
    client.transfer_from(&spender, &owner, &recipient, &150);
}

#[test]
fn test_transfer_from_sequential_calls_preserve_running_state() {
    let (e, _, owner, client) = setup();
    let spender = Address::generate(&e);
    let recipient_one = Address::generate(&e);
    let recipient_two = Address::generate(&e);

    client.mint(&owner, &1000);
    client.approve(&owner, &spender, &700, &1000);
    client.transfer_from(&spender, &owner, &recipient_one, &200);
    client.transfer_from(&spender, &owner, &recipient_two, &300);

    assert_eq!(client.balance(&owner), 500);
    assert_eq!(client.balance(&recipient_one), 200);
    assert_eq!(client.balance(&recipient_two), 300);
    assert_eq!(client.allowance(&owner, &spender), 200);
    assert_eq!(client.supply(), 1000);
}

#[test]
fn test_burn_from() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);

    client.mint(&user1, &1000);
    client.approve(&user1, &user2, &500, &1000);

    client.burn_from(&user2, &user1, &200);

    assert_eq!(client.balance(&user1), 800);
    assert_eq!(client.supply(), 800);
    assert_eq!(client.allowance(&user1, &user2), 300);
}

#[test]
fn test_supply_matches_sum_of_balances_after_mixed_operations() {
    let (e, _, user_one, client) = setup();
    let user_two = Address::generate(&e);
    let user_three = Address::generate(&e);
    let spender = Address::generate(&e);

    client.mint(&user_one, &700);
    client.mint(&user_two, &300);
    client.approve(&user_one, &spender, &150, &1000);
    client.transfer_from(&spender, &user_one, &user_three, &125);
    client.burn(&user_one, &200);

    let total_balances =
        client.balance(&user_one) + client.balance(&user_two) + client.balance(&user_three);
    assert_eq!(client.supply(), total_balances);
}

#[test]
#[should_panic(expected = "balance overflow")]
fn test_balance_overflow() {
    let (_, _, user, client) = setup();
    client.mint(&user, &i128::MAX);
    client.mint(&user, &1);
}

#[test]
#[should_panic(expected = "supply overflow")]
fn test_supply_overflow() {
    let (e, _, user1, client) = setup();
    let user2 = Address::generate(&e);
    client.mint(&user1, &i128::MAX);
    client.mint(&user2, &1);
}

#[test]
#[should_panic(expected = "mint amount must be positive")]
fn test_mint_negative() {
    let (_, _, user, client) = setup();
    client.mint(&user, &-1);
}

#[test]
fn test_transfer_ownership() {
    let (e, _, _, client) = setup();
    let new_admin = Address::generate(&e);

    client.transfer_ownership(&new_admin);

    let user = Address::generate(&e);
    client.mint(&user, &100);
    assert_eq!(client.balance(&user), 100);
}

#[test]
fn test_version_and_status() {
    let (e, _, _, client) = setup();

    assert_eq!(client.version(), String::from_str(&e, "1.0.0"));
    assert_eq!(client.status(), String::from_str(&e, "alive"));
}

#[test]
fn test_set_and_get_metadata_hash() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let token_id = e.register_contract(None, SoroMintToken);
    let client = SoroMintTokenClient::new(&e, &token_id);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&e, "SoroMint"),
        &String::from_str(&e, "SMT"),
    );

    let hash = String::from_str(&e, "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco");
    client.set_metadata_hash(&hash);
    assert_eq!(client.metadata_hash(), Some(hash.clone()));

    let action: Val = symbol_short!("meta_hash").into_val(&e);
    if let Some(event_data) = find_event_by_action(&e, action) {
        let data: (Address, String) = event_data.into_val(&e);
        assert_eq!(data.0, admin);
        assert_eq!(data.1, hash);
    }
}

#[test]
#[should_panic]
fn test_set_metadata_hash_unauthorized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let token_id = e.register_contract(None, SoroMintToken);
    let client = SoroMintTokenClient::new(&e, &token_id);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&e, "SoroMint"),
        &String::from_str(&e, "SMT"),
    );

    client.set_metadata_hash(&String::from_str(&e, "somehash"));
}

proptest! {
    #[test]
    fn prop_version_idempotent(_seed: u64) {
        let e = Env::default();
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let token_id = e.register_contract(None, SoroMintToken);
        let client = SoroMintTokenClient::new(&e, &token_id);
        client.initialize(&admin, &7, &String::from_str(&e, "SoroMint"), &String::from_str(&e, "SMT"));
        prop_assert_eq!(client.version(), client.version());
    }
}

proptest! {
    #[test]
    fn prop_status_idempotent(_seed: u64) {
        let e = Env::default();
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let token_id = e.register_contract(None, SoroMintToken);
        let client = SoroMintTokenClient::new(&e, &token_id);
        client.initialize(&admin, &7, &String::from_str(&e, "SoroMint"), &String::from_str(&e, "SMT"));
        prop_assert_eq!(client.status(), client.status());
    }
}

proptest! {
    #[test]
    fn prop_version_semver_format(_seed: u64) {
        let e = Env::default();
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let token_id = e.register_contract(None, SoroMintToken);
        let client = SoroMintTokenClient::new(&e, &token_id);
        client.initialize(&admin, &7, &String::from_str(&e, "SoroMint"), &String::from_str(&e, "SMT"));
        prop_assert_eq!(client.version(), String::from_str(&e, "1.0.0"));
    }
}

proptest! {
    #[test]
    fn prop_status_is_alive(_seed: u64) {
        let e = Env::default();
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let token_id = e.register_contract(None, SoroMintToken);
        let client = SoroMintTokenClient::new(&e, &token_id);
        client.initialize(&admin, &7, &String::from_str(&e, "SoroMint"), &String::from_str(&e, "SMT"));
        prop_assert_eq!(client.status(), String::from_str(&e, "alive"));
    }
}

proptest! {
    #[test]
    fn prop_no_auth_required(_seed: u64) {
        let e = Env::default();
        let token_id = e.register_contract(None, SoroMintToken);
        let client = SoroMintTokenClient::new(&e, &token_id);
        let _ = client.version();
        let _ = client.status();
    }
}
