# RuNeN - (Rust Neural Network)

<center><img src="https://github.com/greyshaman/runen/raw/refs/heads/dev/images/neuro_mech_3d_l.webp" width="50%" alt="Runen Logo"></center>

Цель этого проекта — создать модель, которая моделирует работу естественной нейронной сети, подобной той, что функционирует в человеческом мозгу. Мы хотим понять, как работает логика в таких нейронных сетях, и изучить различия между искусственными и естественными нейронными сетями в рамках модели.

Для реализации этой модели выбран Rust, так как он обладает богатой экосистемой и множеством преимуществ, таких как высокая производительность, надежные функции безопасности, совместимость с различными платформами и возможность создания хорошо управляемых многозадачных систем.

## Содержание

- [Описание](#описание)
  - [Модель нейросети](#модель-нейросети)
- [Зависимости](#зависимости)
- [Как использовать](#как-использовать)
  - [Создание нейросети](#создание-нейросети)
  - [Создание нейрона](#создание-нейрона)
  - [Соединение нейронов](#соединение-нейронов)
  - [Назначение синапсов в качестве входных портов сети](#назначение-синапсов-в-качестве-входных-портов-сети)
  - [Назначение аксонов в качестве выходных портов сети](#назначение-аксонов-в-качестве-выходных-портов-сети)
- [Чек лист](#чек-лист)

---

## Table of contents

- [Description](#description)
  - [The Neural Network Model](#the-neural-network-model)
- [Dependencies](#dependencies)
- [Howto use](#howto-use)
  - [Creating a neural network](#creating-a-neural-network)
  - [Creating a neuron](#creating-a-neuron)
  - [Connecting neurons](#connecting-neurons)
  - [Assigning synapses as network input ports](#assigning-synapses-as-network-input-ports)
  - [Assigning axons as network output ports](#assigning-axons-as-network-output-ports)
- [Todo](#todo)

## Описание

Распределение и обработка сигналов в сети нейронов - увлекательный и сложный процесс. Эта модель основана на взаимодействиях между биологическими нейронами и процессах, которые происходят между ними.
Значение "u8" представляет собой сигнал, который передается на каждый вход сети. Затем он обрабатывается и может появиться на одном из выходов сети, если только он полностью не подавлен или не отправлен через внутренние циклы.
В действительности сигнал можно рассматривать как волну возбуждения, которая течет от входа к выходу, с возможными ответвлениями на этом пути.

### Модель нейросети

Нейрон является основным строительным блоком нейронной сети. Он состоит из дендритов с синапсами, которые действуют как входные порты, принимающие сигналы от других нейронов. Затем эти сигналы обрабатываются в клеточном теле нейрона, где накапливается информация. Аксон, в свою очередь, посылает обработанный сигнал другим нейронам через синапсы. Аксон может быть соединен с одним или несколькими другими нейронами в сети, создавая путь для передачи информации.

Синапс может быть активирован, если он имеет соединение с аксоном какого-либо нейрона. Синапс имеет три параметра: максимальная ёмкость, текущая ёмкость и величина регенерации ёмкости перед принятием следующего сигнала. При поступлении к синапсу входного сигнала он может возбудиться на величину сигнала, но не больше текущей ёмкости. Таким образом, сигнал ограничивается величиной ёмкости. Далее ёмкость синапса уменьшается на величину сигнала, но за счёт регенерации восстанавливается на величину регенерации к приходу следующего сигнала.

Дендрит, принимающий сигнал, прошедший через ограничение, имеет параметр веса. Вес может быть положительным (возбуждение) или отрицательным (торможение), и значение сигнала умножается на этот вес.

Нейрон асинхронно обрабатывает сигнал от дендрита и объединяет его с другими сигналами, полученными от других дендритов. Это накопление происходит до того, как нейрон сбрасывается, что происходит, когда поступают все сигналы от активированных аксонов или когда повторный сигнал поступает на дендрит. Когда нейрон сбрасывает данные из сумматора, он передает накопленное значение аксону и записывает единичное значение в сумматор. Таким образом, даже если нейрон получает нулевой входящий сигнал, он все равно выдает сигнал при сбросе.

Аксон передает положительный сигнал, полученный от сумматора, во все подключенные к нему синапсы.

Таким образом, нейроны обмениваются сигналами.

Сеть содержит в себе набор нейронов и управляет созданием, конфигурацией и уничтожением нейронов. Для получения сигналов сеть имеет входные порты, которые соединяются с определёнными синапсами нейронов, и выходные порты (соединяются с некоторыми аксонами), c которых можно снять обработанные сигналы.

## Зависимости

Для реализации проекта применялись:

- Rust (1.82.0)
- regex (1.11.1)
- tokio (1.41.1)

## Как использовать

Чтобы использовать нейронную сеть, до реализации процесса обучения, сеть необходимо настроить вручную.

### Создание нейросети

Нейросеть создаётся при помощи конструктора `Network::new()`:

```rust
use librunen::rnn::layouts::network::Network;

let net = Network::new();
....
```

### Создание нейрона

Сеть имеет асинхронный метод для создания и добавления в свой состав нейронов, которая возвращает результат со ссылкой на созданный нейрон или ошибку если она возникла:

`async fn Network::create_neuron(&self, net: Arc<Network>, input_cfg: Vec<InputCfg>) -> Result<Arc<Neuron>, Box<dyn Error>>`

При создании нейрону необходимо предоставить ссылку на сеть к которой он будет принадлежать и конфигурацию его входного интерфейса в виде вектора `Vec<InputCfg>`.
Если передать пустой вектор, то создастся нейрон с одним входом (синапс с ёмкостью = 1 и регенерацией на 1, и дендрит с весом равным 1).

```rust
let net = Arc::new(Network::new());
let neuron_input_cfg = vec![];
let neuron = net.create_neuron(net.clone(), neuron_input_cfg)
  .await
  .unwrap();
```

В случае когда нужно сразу указать несколько входов это можно сделать так:

```rust
.... create net instance ....

let neuron_input_cfg = vec![
  InputCfg::new(1, 1, -1).unwrap(),
  InputCfg::new(2, 2, 1).unwrap(),
  InputCfg::new(1, 1, 1).unwrap()
];
let neuron = net.create_neuron(net.clone(), neuron_input_cfg)
  .await
  .unwrap();

....
```

### Соединение нейронов

Сеть имеет асинхронный метод который соединяет указанные нейроны:

`async fn Network::connect_neurons(&self, src_id: &str, dst_id: &str, dst_port: usize) -> Result<(), Box<dyn Error>>`

Аксон нейрона с идентификатором `src_id` соединяется со свободным синапсом `dst_port` дендрита нейрона с идентификатором `dst_id`. Если указаные нейроны не будут найдены или синапс занят, метод вернёт ошибку:

```rust
.... create net instance ....

let neuron_1 = net.create_neuron(net.clone(), vec![]).await.unwrap();
let src_id = neuron_1.get_id();

let neuron_2 = net.create_neuron(net.clone(), vec![
  InputCfg::new(1, 1, 1).unwrap(),
  InputCfg::new(2, 1, -1).unwrap()
])
  .await
  .unwrap();
let dst_id = neuron_2.get_id();

assert!(net.connect_neurons(&src_id, &dst_id, 1).await.is_ok());
```

### Назначение синапсов в качестве входных портов сети

Конечные (терминальные) нейроны из входного слоя, могут получать сигналы поступающие на входные порты сети. Для этого их синапсы связываются с этими портами. Сеть имеет асинхронный метод для этого:

`asyn fn Network::setup_input(&self, network_port: usize, neuron_id: &str, neuron_port: usize) -> Result<(), Box<(dyn Error)>>`

Здесь `network_port` - номер входного порта, `neuron_id` - идентификатор нейрона, `neuron_port` - номер дендрита с синапсом, который связывается с входным портом.

```rust
let net = Arc::new(Network::new());

let neuron = net.create_neuron(net.clone(), vec![
  InputCfg::new(1, 1, 1).unwrap(),
  InputCfg::new(2, 1, -1).unwrap()
])
  .await
  .unwrap();
let id = neuron.get_id();

// assign second neuron's input as first network's input port
assert!(net.setup_input(0, &id, 1).await.is_ok());

```

### Назначение аксонов в качестве выходных портов сети

Конечные (термнальные) нейроны из выходного слоя, могут передавать сигналы на внешние выходные порты нейросети. Для этого их аксоны связываются с этими портами. Сеть имеет асинхронный метод для этого:

`async fn setup_output(&self, network_port: usize, neuron_id: &str) -> Result<(), Box<dyn Error>>`

Здесь `network_port` - номер выходного порта, `neuron_id` - идентификатор нейрона аксон которого передаёт обработаный сигнал за пределы сети через порт вывода.

```rust
let net = Arc::new(Network::new());

.... create neurons for other layers ....

// create terminal output neuron
let out_neuron = net.create_neuron(net.clone(), vec![]).await.unwrap();
let out_id = out_neuron.get_id();

// assign axon of out_neuron as output port
assert!(net.setup_output(0, &out_id).await.is_ok());

let net_clone = net.clone();

// read signal from output port
let jh0 = tokio::task::spawn(async move{
  let out_port_0: Arc<RwLock<Receiver<u8>>> = net_clone.get_output_receiver(0)
    .await
    .unwrap()
  let mut rx = out_port_0.write().await;
  while let Ok(signal) = rx.recv().await {
    // use or store received signal here
    ....
  }
});

....
```

В этом фрагменте аксон одного из нейронов назначается как первый порт вывода с индексом 0. Сеть сопоставляет номерам портов ссылки на защищённые `RwLock` получатели `Arc<RwLock<Receiver<u8>>>` из которого в отдельной задаче читаются обработаные сигналы. Эти сигналы в дальнейшем могут быть использованы в зависимости от цели использования нейросети.

## Чек лист

- [x] ~~разработать модель биологической нейросети.~~
- [x] ~~сделать синхронную реализацию модели.~~
- [x] ~~Сделать асинхронную реализацию модели~~
- [x] ~~Добавить юнит тесты.~~
- [ ] Добавить интеграционные тесты.
- [ ] Улучшить взаимодействие с интерфейсами нейросети.
- [ ] Добавить бенчмарки и профилировать код.
- [ ] Добавить чтение/запись конфигурации: использовать serde.
- [ ] Реализовать обучение сети.
- [ ] Разработать управляющую систему, которая будет управлять сетями (Создание, обучение, взаимодействие сетей).
- [ ] Визуализировать процесс работы сети.
- [ ] Добавить реализацию нейронной сети, использующую тензорные представления данных.

---

The goal of this project is to create a model that simulates the operation of a natural neural network similar to the one that functions in the human brain. We want to understand how logic works in such neural networks and explore the differences between artificial and natural neural networks within the framework of the model.

Rust was chosen to implement this model, as it has a rich ecosystem and many advantages.
These include high performance, robust security features, and compatibility with various platforms.
It is very interesting how Rust will cope with this task, but success will also depend on the programmer's ability to build the model correctly.

## Description

Signal distribution and processing within a network of neurons is a fascinating and complex process. This model is based on the interactions between biological neurons and the processes that occur between them.
The value `u8` represents a signal that is transmitted to each input of the network. It is then processed and may appear on one of the network's outputs, unless it is completely suppressed or sent through internal cycles.
In reality, the signal can be thought of as an excitation wave that flows from the input towards the output, with possible branches along the way.

### The Neural Network Model

A neuron is the basic building block of a neural network. It consists of dendrites with synapses that act as input ports receiving signals from other neurons. These signals are then processed in the cell body of the neuron, where information is accumulated. The axon, in turn, sends the processed signal to other neurons through synapses. An axon can be connected to one or more other neurons in the network, creating a pathway for information transmission.

A synapse can be activated if it has a connection to the axon of a neuron. The synapse has three parameters: maximum capacity, current capacity, and the amount of capacity regeneration before receiving the next signal. When an input signal arrives at the synapse, it can be excited by the value of the signal, but not more than the current capacity. Thus, the signal is limited by the amount of capacity. Further, the synapse capacity decreases by the amount of the signal, but due to regeneration it is restored by the amount of regeneration by the arrival of the next signal.

The dendrite that receives the signal that has passed through the restriction has a weight parameter. The weight can be positive (arousal) or negative (inhibition), and the signal value is multiplied by this weight.

The neuron asynchronously processes the signal from the dendrite and combines it with other signals received from other dendrites. This accumulation occurs before the neuron is reset, which happens when all signals from activated axons arrive or when a repeated signal arrives at the dendrite. When the neuron resets data from the adder, it transmits the accumulated value to the axon and writes a single value to the adder. Thus, even if the neuron receives a zero incoming signal, it still outputs a signal when reset.

The axon transmits the positive signal received from the adder to all the synapses connected to it.

The network contains a set of neurons and controls the creation, configuration and destruction of neurons. To receive signals, the network has input ports that connect to certain synapses of neurons, and output ports (connect to some axons) from which processed signals can be removed.

## Dependencies

- Rust (1.82.0)
- regex (1.11.1)
- tokio (1.41.1)

## Howto use

To use a neural network, before implementing the learning process, the network must be configured manually.

### Creating a neural network

A neural network is created using the `Network::new()` constructor:

```rust
use librunen::rnn::layouts::network::Network;

let net = Network::new();
....
```

### Creating a neuron

The network has an asynchronous method for creating and adding neurons to its composition, which returns a result with a link to the created neuron or an error if it occurred:

`async fn Network::create_neuron(&self, net: Arc<Network>, input_cfg: Vec<InputCfg>) -> Result<Arc<Neuron>, Box<dyn Error>>`

When creating a neuron, it is necessary to provide a link to the network to which it will belong and the configuration of its input interface in the form of the vector `Vec<InputCfg>`.
If you pass an empty vector, a neuron with one input will be created (a synapse with capacity = 1 and regeneration by 1, and a dendrite with weight equal to 1).

```rust
let net = Arc::new(Network::new());
let neuron_input_cfg = vec![];
let neuron = net.create_neuron(net.clone(), neuron_input_cfg)
  .await
  .unwrap();
```

In the case when you need to specify several inputs at once, you can do this:

```rust
.... create net instance ....

let neuron_input_cfg = vec![
  InputCfg::new(1, 1, -1).unwrap(),
  InputCfg::new(2, 2, 1).unwrap(),
  InputCfg::new(1, 1, 1).unwrap()
];
let neuron = net.create_neuron(net.clone(), neuron_input_cfg)
  .await
  .unwrap();

....
```

### Connecting neurons

The network has an asynchronous method that connects the specified neurons:

`async fn Network::connect_neurons(&self, src_id: &str, dst_id: &str, dst_port: usize) -> Result<(), Box<dyn Error>>`

The axon of the neuron with the identifier `srs_id` connects to the free synapse `dst_port` of the dendrite of the neuron with the identifier `dst_id`. If the specified neurons are not found or the synapse is occupied, the method returns an error:

```rust
.... create net instance ....

let neuron_1 = net.create_neuron(net.clone(), vec![]).await.unwrap();
let src_id = neuron_1.get_id();

let neuron_2 = net.create_neuron(net.clone(), vec![
  InputCfg::new(1, 1, 1).unwrap(),
  InputCfg::new(2, 1, -1).unwrap()
])
  .await
  .unwrap();
let dst_id = neuron_2.get_id();

assert!(net.connect_neurons(&src_id, &dst_id, 1).await.is_ok());
```

### Assigning synapses as network input ports

Terminal neurons from the input layer can receive signals coming to the input ports of the network. To do this, their synapses bind to these ports. The network has an asynchronous method for this:

`async fn Network::setup_input(&self, network_port: size, neuron_id: &str, neuron_port: size) -> Result<(), Box<(dir Error)>>`

Here `network_port` is the number of the input port, `neuron_id` is the identifier of the neuron, `neuron_port` is the number of the dendrite with the synapse that binds to the input port.

```rust
let net = Arc::new(Network::new());

let neuron = net.create_neuron(net.clone(), vec![
  InputCfg::new(1, 1, 1).unwrap(),
  InputCfg::new(2, 1, -1).unwrap()
])
  .await
  .unwrap();
let id = neuron.get_id();

// assign second neuron's input as first network's input port
assert!(net.setup_input(0, &id, 1).await.is_ok());

```

### Assigning axons as network output ports

Terminal neurons from the output layer can transmit signals to the external output ports of the neural network. To do this, their axons bind to these ports. The network has an asynchronous method for this:

`async fn setup_output(&self, network_port: usize, neuron_id: &str) -> Result<(), Box<dyn Error>>`

Here `network_port` is the number of the output port, `neuron_id` is the identifier of the neuron whose axon transmits the processed signal outside the network through the output port.

```rust
let net = Arc::new(Network::new());

.... create neurons for other layers ....

// create terminal output neuron
let out_neuron = net.create_neuron(net.clone(), vec![]).await.unwrap();
let out_id = out_neuron.get_id();

// assign axon of out_neuron as output port
assert!(net.setup_output(0, &out_id).await.is_ok());

let net_clone = net.clone();

// read signal from output port
let jh0 = tokio::task::spawn(async move{
  let out_port_0: Arc<RwLock<Receiver<u8>>> = net_clone.get_output_receiver(0)
    .await
    .unwrap()
  let mut rx = out_port_0.write().await;
  while let Ok(signal) = rx.recv().await {
    // use or store received signal here
    ....
  }
});

....
```

In this fragment, the axon of one of the neurons is assigned as the first output port with the index 0. The network maps port numbers to links to protected `RwLock` receivers `Arc<RwLock<Receiver<u8>>>` from which processed signals are read in a separate task. These signals can later be used depending on the purpose of using the neural network.

## Todo

- [x] ~~develop a model of a biological neural network.~~
- [x] ~~make a synchronous implementation of the model.~~
- [x] ~~To make an asynchronous implementation of the model~~
- [x] ~~Add unit tests.~~
- [ ] Add integration tests.
- [ ] Improve interaction with neural network interfaces.
- [ ] Add benchmarks and profile the code.
- [ ] Add read/write configuration: use serge.
- [ ] Implement network training.
- [ ] Develop a management system that will manage networks (Creation, training, networking).
- [ ] Visualize the network operation process.
- [ ] Add a neural network implementation using tensor representations of data.
