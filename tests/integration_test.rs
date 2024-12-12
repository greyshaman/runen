use std::{sync::Arc, time::Duration};

use librunen::rnn::{common::input_cfg::InputCfg, layouts::network::Network};
use tokio::time::sleep;

#[tokio::test]
async fn test_signal_propagation() {
    let net = Arc::new(Network::new().unwrap());

    let var_name = vec![
        InputCfg::new(2, 2, -1).unwrap(),
        InputCfg::new(1, 1, 1).unwrap(),
    ];
    let config1 = var_name;

    let config2 = vec![
        InputCfg::new(1, 1, -2).unwrap(),
        InputCfg::new(2, 2, 1).unwrap(),
    ];

    let neuron0 = net.create_neuron(net.clone(), vec![]).await.unwrap();
    let id0 = neuron0.get_id();
    let neuron1 = net.create_neuron(net.clone(), config1).await.unwrap();
    let id1 = neuron1.get_id();
    let neuron2 = net.create_neuron(net.clone(), config2).await.unwrap();
    let id2 = neuron2.get_id();

    // create inter neuron links
    assert!(net.connect_neurons(&id0, &id1, 0).await.is_ok());
    assert!(net.connect_neurons(&id0, &id1, 1).await.is_ok());

    assert!(net.connect_neurons(&id0, &id2, 0).await.is_ok());
    assert!(net.connect_neurons(&id0, &id2, 1).await.is_ok());

    // link neuron's synapse with network's input port
    assert!(net.setup_input(0, &id0, 0).await.is_ok());

    // link neurons' axons with network's output ports
    assert!(net.setup_output(0, &id1).await.is_ok());
    assert!(net.setup_output(1, &id2).await.is_ok());

    // input signal 0 into 0 network's port
    assert!(net.input(0, 0).await.is_ok());
    sleep(Duration::from_millis(1)).await;

    // input signal 1 into 0 network's port
    assert!(net.input(1, 0).await.is_ok());
    sleep(Duration::from_millis(1)).await;

    let state0 = net
        .get_current_neuron_statistics(&neuron0.get_id())
        .await
        .unwrap();
    assert_eq!(state0.hit_count, 2);

    // let results = net.pop_result_log().await;
    // assert_eq!(results.len(), 2);
}
