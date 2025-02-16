use fuel_core::{
    chain_config::{
        CoinConfig,
        StateConfig,
    },
    service::{
        Config,
        FuelService,
    },
};
use fuel_core_interfaces::common::{
    fuel_tx::TransactionBuilder,
    fuel_vm::prelude::*,
};
use fuel_gql_client::client::FuelClient;
use rand::{
    rngs::StdRng,
    Rng,
    SeedableRng,
};
use std::time::Duration;

fn create_node_config_from_inputs(inputs: &[Input]) -> Config {
    let mut node_config = Config::local_node();
    let mut initial_state = StateConfig::default();
    let mut coin_configs = vec![];

    for input in inputs {
        if let Input::CoinSigned {
            amount,
            owner,
            asset_id,
            utxo_id,
            ..
        }
        | Input::CoinPredicate {
            amount,
            owner,
            asset_id,
            utxo_id,
            ..
        } = input
        {
            let coin_config = CoinConfig {
                tx_id: Some(*utxo_id.tx_id()),
                output_index: Some(utxo_id.output_index() as u64),
                block_created: None,
                maturity: None,
                owner: *owner,
                amount: *amount,
                asset_id: *asset_id,
            };
            coin_configs.push(coin_config);
        };
    }

    initial_state.coins = Some(coin_configs);
    node_config.chain_conf.initial_state = Some(initial_state);
    node_config.utxo_validation = true;
    node_config.p2p.enable_mdns = true;
    node_config
}

#[tokio::test]
async fn test_tx_gossiping() {
    let mut rng = StdRng::seed_from_u64(2322);

    let tx = TransactionBuilder::script(vec![], vec![])
        .gas_limit(100)
        .gas_price(1)
        .add_unsigned_coin_input(
            SecretKey::random(&mut rng),
            rng.gen(),
            1000,
            Default::default(),
            Default::default(),
            0,
        )
        .add_output(Output::Change {
            amount: 0,
            asset_id: Default::default(),
            to: rng.gen(),
        })
        .finalize();

    let node_config = create_node_config_from_inputs(tx.inputs());
    let node_one = FuelService::new_node(node_config).await.unwrap();
    let client_one = FuelClient::from(node_one.bound_address);

    let node_config = create_node_config_from_inputs(tx.inputs());
    let node_two = FuelService::new_node(node_config).await.unwrap();
    let client_two = FuelClient::from(node_two.bound_address);

    tokio::time::sleep(Duration::new(3, 0)).await;

    let result = client_one.submit(&tx).await.unwrap();

    let tx = client_one.transaction(&result.0.to_string()).await.unwrap();
    assert!(tx.is_some());

    tokio::time::sleep(Duration::new(3, 0)).await;

    let tx = client_two.transaction(&result.0.to_string()).await.unwrap();
    assert!(tx.is_some());
}
