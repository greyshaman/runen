# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [0.4.0]

### Added

- Added Processors: Stdp, Svtdp and RCSA. They are implementing different signal processing logic.
- Signal has customized data type and creation time.
- Added layers (input, hidden, output).
- Added input and output ports.
- Added synapse trait.
- Added new error types into RnnError: BadConfig, SendingWithoutConnection.

### Changed

- Changed neuron structure: moved out signal processing logic to Processors entities
- Changed neural network structure
- Changed status to use new signal structure.

### Removed

- The outdated signal_handlers has been removed.
- The outdated input port configuration has been removed.
- The outdated dendrite has been removed.

## [0.2.0]

### Added

- Implementation network configuration format and its serialization/deserialization.
- Added integration test for network.
- Added serde_json(v1.0.133) to allow serialization into json format.
- Added serde_yaml(v0.9.34) to allow serialization into yaml format.
- Added chrono(v0.4.38) to allow to fix time moment for monitoring records.
- Added command broadcast channel to send command from network to neurons. Implemented switch_monitoring_mode command.
- Implementation of monitoring neuron activity (when signal receiving).
- Added bias parameter for neuron.

### Changed

- Change Network structure: added commands channel and improve monitoring channel.
- Hide neuron constructor (Neuron::new) into private scope. Use Neuron::build to create neuron from now.
- Extend Neuron::receive() args list.
- Changed monitoring statistics to component status.
- Change error types.

### Removed

- Removed method Network::activate_trace_log()

## [0.1.3] - 2024-11-19

### Added

- Added more error types.
- Wrapped channel readier threads into TaskTracker.
- Handling error on receiving and sending signals.

### Changed

- Added errors handling on InputCfg constructor.
- Changed contract for neuron builder function (arguments swapped).
- Improved code readability: changed input to synapse.
- Improved code readability: set more meaningful name to private method.

## [0.1.2] - 2024-11-17

### Added

- Added network's work modes: Standard, Trace.

### Changed

- Added more section in Readme.

## [0.1.1] - 2024-11-16

### Added

- Added new sections to readme file.
- Added MIT license.

### Changed

- Improved code: Use RwLocks instead of Mutexes.

## [0.1.0] - 2024-11-15

### Added

- The tokio (v1.41.1) dependency added: change to asynchronously signal operation.
- The tokio-stream (v0.1.16) dependency added: used to testing signal generation.
- Added simple library usage example

### Changed

- Huge reworked project: removed separate parts of neuron and injected them functionality into neuron
- Changed error type arg for RnnError::IdBusy.
- Changed neuron creating logic.

### Removed

- Removed reflection like functions to downcast abstract entities.

## [0.0.3] - 2024-11-07

### Changed

- Changed components hierarchy.

## [0.0.1] - 2024-10-19

### Added

- Started project
- Added synchronous model implementation
