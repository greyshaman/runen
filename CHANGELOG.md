# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [Unreleased]

### Added

- Implementation network configuration format and its serialization/deserialization.
- Added integration test for network.
- Added serde_json(v1.0.133) to allow serialization into json format.
- Added serde_yaml(v0.9.34) to allow serialization into yaml format.

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
