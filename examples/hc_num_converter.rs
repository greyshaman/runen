//! This is an example of using a neural network with a fixed structure
//! to test whether it is possible to convert a binary two-bit number
//! into a decimal number.

use std::sync::Arc;
use std::time::Duration;

use librunen::rnn::common::signal::{self, Signal};
use librunen::rnn::layouts::input_port::InputPort;
use librunen::rnn::layouts::network::Network;
use librunen::rnn::layouts::output_port::OutputPort;
use librunen::rnn::neural::neuron::Neuron;
use tokio::task;
use tokio::time::sleep;

async fn generate_net(net: Arc<Network<i16>>) {
    // The M0Z0 neuron
    // let neuron0: Arc<Neuron<i16>> = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();

    // The M0Z1 neuron
    // let neuron1 = net
    //     .create_neuron(
    //         net.clone(),
    //         1,
    //         vec![
    //             InputCfg::new(2, 2, -1, 0).unwrap(),
    //             InputCfg::new(1, 1, 1, 0).unwrap(),
    //         ],
    //     )
    //     .await
    //     .unwrap();

    // // The M0Z2 neuron
    // let neuron2 = net
    //     .create_neuron(
    //         net.clone(),
    //         1,
    //         vec![
    //             InputCfg::new(1, 1, -2, 0).unwrap(),
    //             InputCfg::new(2, 2, 1, 0).unwrap(),
    //         ],
    //     )
    //     .await
    //     .unwrap();

    // let id0 = neuron0.get_id();
    // let id1 = neuron1.get_id();
    // let id2 = neuron2.get_id();

    // assert!(net.connect_neurons(&id0, &id1, 0).await.is_ok());
    // assert!(net.connect_neurons(&id0, &id1, 1).await.is_ok());

    // assert!(net.connect_neurons(&id0, &id2, 0).await.is_ok());
    // assert!(net.connect_neurons(&id0, &id2, 1).await.is_ok());

    // // Config input
    // assert!(net.setup_input(0, "M0Z0", 0).await.is_ok());

    // // Config output
    // assert!(net.setup_output(0, "M0Z1").await.is_ok());
    // assert!(net.setup_output(1, "M0Z2").await.is_ok());
}

#[tokio::main]
async fn main() {
    let net = Arc::new(Network::new().unwrap());
    generate_net(net.clone()).await;

    let net_clone = net.clone();

    // let _t1 = task::spawn(async move {
    //     let zero_signal = net_clone.get_output_receiver(0).await.unwrap();
    //     let mut rx = zero_signal.write().await;
    //     while let Ok(signal) = rx.recv().await {
    //         let value = match signal.value() {
    //             SignalType::Spike => 1,
    //             SignalType::Stockade(val) => val,
    //         };
    //         println!("-+= 0 =+- ({})", value);
    //     }
    // });

    // let net_clone = net.clone();

    // let _t2 = task::spawn(async move {
    //     let one_signal = net_clone.get_output_receiver(1).await.unwrap();
    //     let mut rx = one_signal.write().await;
    //     while let Ok(signal) = rx.recv().await {
    //         let value = match signal.value() {
    //             SignalType::Spike => 1,
    //             SignalType::Stockade(val) => val,
    //         };
    //         println!("-+= 1 =+- ({})", value);
    //     }
    // });

    // println!("Sending 0 at first bit");
    // assert!(
    //     net.input(Signal::new(SignalType::Stockade(0)), 0)
    //         .await
    //         .is_ok()
    // );
    // sleep(Duration::from_millis(1)).await;

    // println!("Sending 1 at first bit");
    // assert!(
    //     net.input(Signal::new(SignalType::Stockade(1)), 0)
    //         .await
    //         .is_ok()
    // );
    // sleep(Duration::from_millis(1)).await;

    // let results = net.pop_result_log().await;
    // println!("results: {:?}", results);
}
