//! This is an example of using a neural network with a fixed structure
//! to test whether it is possible to convert a binary two-bit number
//! into a decimal number.

use std::sync::Arc;
use std::time::Duration;

use librunen::rnn::common::input_cfg::InputCfg;
use librunen::rnn::layouts::network::Network;
use tokio::task;
use tokio::time::sleep;

async fn generate_net(net: Arc<Network>) {
    // The M0Z0 neuron
    let neuron0 = net.create_neuron(net.clone(), vec![]).await.unwrap();

    // The M0Z1 neuron
    let neuron1 = net
        .create_neuron(
            net.clone(),
            vec![
                InputCfg::new(2, 2, -1).unwrap(),
                InputCfg::new(1, 1, 1).unwrap(),
            ],
        )
        .await
        .unwrap();

    // The M0Z2 neuron
    let neuron2 = net
        .create_neuron(
            net.clone(),
            vec![
                InputCfg::new(1, 1, -2).unwrap(),
                InputCfg::new(2, 2, 1).unwrap(),
            ],
        )
        .await
        .unwrap();

    let id0 = neuron0.get_id();
    let id1 = neuron1.get_id();
    let id2 = neuron2.get_id();

    assert!(net.connect_neurons(&id0, &id1, 0).await.is_ok());
    assert!(net.connect_neurons(&id0, &id1, 1).await.is_ok());

    assert!(net.connect_neurons(&id0, &id2, 0).await.is_ok());
    assert!(net.connect_neurons(&id0, &id2, 1).await.is_ok());

    // Config input
    assert!(net.setup_input(0, "M0Z0", 0).await.is_ok());

    // Config output
    assert!(net.setup_output(0, "M0Z1").await.is_ok());
    assert!(net.setup_output(1, "M0Z2").await.is_ok());
}

#[tokio::main]
async fn main() {
    let net = Arc::new(Network::new().unwrap());
    generate_net(net.clone()).await;

    let net_clone = net.clone();

    let _t1 = task::spawn(async move {
        let zero_signal = net_clone.get_output_receiver(0).await.unwrap();
        let mut rx = zero_signal.write().await;
        while let Ok(signal) = rx.recv().await {
            println!("-+= 0 =+- ({})", signal);
        }
    });

    let net_clone = net.clone();

    let _t2 = task::spawn(async move {
        let one_signal = net_clone.get_output_receiver(1).await.unwrap();
        let mut rx = one_signal.write().await;
        while let Ok(signal) = rx.recv().await {
            println!("-+= 1 =+- ({})", signal);
        }
    });

    {}

    println!("Sending 0 at first bit");
    assert!(net.input(0, 0).await.is_ok());
    sleep(Duration::from_millis(1)).await;

    println!("Sending 1 at first bit");
    assert!(net.input(1, 0).await.is_ok());
    sleep(Duration::from_millis(1)).await;

    // let results = net.pop_result_log().await;
    // println!("results: {:?}", results);
}
