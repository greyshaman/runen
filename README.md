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
- [Чек лист](#чек-лист)

---

## Table of contents

- [Description](#description)
  - [The Neural Network Model](#the-neural-network-model)
- [Dependencies](#dependencies)
- [Howto use](#howto-use)
  - [Creating a neural network](#creating-a-neural-network)
- [Todo](#todo)

## Описание

Распределение и обработка сигнала сетью нейронов — очень интересный и сложный процесс. В основе модели лежит механизм взаимодействия биологических нейронов и процессов между ними.

### Модель нейросети

Нейрон является основным строительным блоком нейронной сети. У него есть дендриты, которые действуют как входные порты, принимая сигналы от других нейронов. Эти сигналы обрабатываются в центральном теле нейрона, где накапливается информация. Затем аксон посылает полученный сигнал другим нейронам через синапсы. Аксон может быть соединен с одним или несколькими нейронами в сети, образуя путь для передачи информации.

Синапс может быть активирован, если он имеет соединение с аксоном какого-либо нейрона. Синапс имеет три параметра: максимальная ёмкость, текущая ёмкость и величина регенерации ёмкости перед принятием следующего сигнала. При поступлении к синапсу входного сигнала он может возбудиться на величину сигнала, но не больше текущей ёмкости. Таким образом, сигнал ограничивается величиной ёмкости. Далее ёмкость синапса уменьшается на величину сигнала, но за счёт регенерации восстанавливается на величину регенерации.

Дендрит, принимающий сигнал, прошедший через ограничение, имеет параметр веса. Вес может быть положительным (возбуждение) или отрицательным (торможение), и значение сигнала умножается на этот вес.

Нейрон асинхронно обрабатывает сигнал от дендрита и объединяет его с другими сигналами, полученными от других дендритов. Это накопление происходит до того, как нейрон сбрасывается, что происходит, когда поступают все сигналы от активированных аксонов или когда повторный сигнал поступает на дендрит. Когда нейрон сбрасывает данные из сумматора, он передает накопленное значение аксону и записывает единичное значение в сумматор. Таким образом, даже если нейрон получает нулевой входящий сигнал, он все равно выдает сигнал при сбросе.

Аксон передает положительный сигнал, полученный от сумматора, во все подключенные к нему синапсы.

Таким образом, нейроны обмениваются сигналами.

Сеть содержит в себе набор нейронов и управляет созданием, конфигурацией и уничтожением нейронов. Для получения сигналов сеть имеет входные порты, которые соединяются с определёнными синапсами нейронов, и выходные порты, c которых можно снять обработанные сигналы.

## Зависимости

Для реализации проекта применялись:

- Rust (1.82.0)
- regex (1.11.1)
- tokio (1.41.1)

## Как использовать

Чтобы использовать нейронную сеть, до реализации процесса обучения сеть необходимо настроить вручную.

### Создание нейросети

... В процессе создания ...

## Чек лист

- [x] ~~разработать модель биологической нейросети.~~
- [x] ~~сделать синхроyную реализацию модели.~~
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

The distribution and processing of signals by a network of neurons is an interesting and complex process. This model is based on the interaction between biological neurons and the processes that occur between them.

### The Neural Network Model

A neuron is the basic building block of a neural network. It has dendrites that act as input ports, receiving signals from other neurons. These signals are processed in the central body of the neuron, where information is accumulated. The axon then sends the resulting signal to other neurons through synapses. An axon can be connected to one or more neurons in the network, forming a path for information to flow.

A synapse can be activated if it has a connection to the axon of a neuron. The synapse has three parameters: maximum capacity, current capacity, and the amount of capacity regeneration before receiving the next signal. When an input signal arrives at the synapse, it can be excited by the value of the signal, but not more than the current capacity. Thus, the signal is limited by the amount of capacitance. Further, the capacity of the synapse decreases by the amount of the signal, but due to regeneration it is restored by the amount of regeneration.

The dendrite that receives the signal that has passed through the restriction has a weight parameter. The weight can be positive (arousal) or negative (inhibition), and the signal value is multiplied by this weight.

The neuron asynchronously processes the signal from the dendrite and combines it with other signals received from other dendrites. This accumulation occurs before the neuron is reset, which happens when all signals from activated axons arrive or when a repeated signal arrives at the dendrite. When the neuron resets data from the adder, it transmits the accumulated value to the axon and writes a single value to the adder. Thus, even if the neuron receives a zero incoming signal, it still outputs a signal when reset.

The axon transmits the positive signal received from the adder to all the synapses connected to it.

## Dependencies

- Rust (1.82.0)
- regex (1.11.1)
- tokio (1.41.1)

## Howto use

To use a neural network, the network must be configured manually before the learning process is implemented.

### Creating a neural network

... under construction ...


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
