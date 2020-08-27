use crate::tests::*;
use crate::contracts::{
    EncodedMessage,
    deploy::{DeployFunctionCallSet, ParamsOfDeploy}
};
use super::*;

#[test]
fn block_signatures() {
    let client = TestClient::new();

    let _: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "blocks_signatures".to_owned(),
            filter: Some(json!({})),
            result: "id".to_owned(),
            limit: Some(1),
            order: None,
        }
    ).unwrap();
}

#[test]
fn all_accounts() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({})),
            result: "id balance".to_owned(),
            limit: None,
            order: None,
        }
    ).unwrap();

    assert!(accounts.result.len() > 0);
}

#[test]
fn ranges() {
    let client = TestClient::new();

    let accounts: ResultOfQueryCollection = client.request(
        "queries.query_collection",
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(json!({
                "created_at": { "gt": 1562342740 }
            })),
            result: "body created_at".to_owned(),
            limit: None,
            order: None,
        }
    ).unwrap();

    assert!(accounts.result[0]["created_at"].as_u64().unwrap() > 1562342740);
}

#[test]
fn wait_for() {
    let handle = std::thread::spawn(|| {
        let client = TestClient::new();
        let now = ton_sdk::Contract::now();
        let transactions: ResultOfWaitForCollection = client.request(
            "queries.wait_for_collection",
            ParamsOfWaitForCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "now": { "gt": now }
                })),
                result: "id now".to_owned(),
                timeout: None
            }
        ).unwrap();

        assert!(transactions.result["now"].as_u64().unwrap() > now as u64);
    });

    let client = TestClient::new();

    client.get_grams_from_giver(&TestClient::get_giver_address(), None);

    handle.join().unwrap();
}

#[test]
fn subscribe_for_transactions_with_addresses() {
    let client = TestClient::new();
    let keys = client.generate_kepair();
    let deploy_params = ParamsOfDeploy{
        call_set: DeployFunctionCallSet {
            abi: HELLO_ABI.clone(),
            constructor_header: None,
            constructor_params: json!({}),
        },
        image_base64: base64::encode(HELLO_IMAGE.as_slice()),
        init_params: None,
        key_pair: keys,
        workchain_id: None,
        try_index: None
    };

    let msg: EncodedMessage = client.request(
        "contracts.deploy.message",
        deploy_params.clone()
    ).unwrap();

    let handle: ResultOfSubscribeCollection = client.request(
            "queries.subscribe_collection",
            ParamsOfSubscribeCollection {
                collection: "transactions".to_owned(),
                filter: Some(json!({
                    "account_addr": { "eq": msg.address.clone().unwrap() },
                    "status": { "eq": ton_sdk::json_helper::transaction_status_to_u8(ton_block::TransactionProcessingStatus::Finalized) }
                })),
                result: "id account_addr".to_owned(),
            }
        ).unwrap();

    client.deploy_with_giver(deploy_params, None);

    let mut transactions = vec![];

    for _ in 0..2 {
        let result: ResultOfGetNextSubscriptionData = client.request(
            "queries.get_next_subscription_data", handle.clone()).unwrap();
        assert_eq!(result.result["account_addr"], msg.address.clone().unwrap());
        transactions.push(result.result);
    }

    assert_ne!(transactions[0]["id"], transactions[1]["id"]);

    let _: () = client.request("queries.unsubscribe", handle).unwrap();
}