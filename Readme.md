# Nodes Hub

A collection of [Peppy](https://github.com/Peppy-bot/peppy) nodes for robotic systems. Each node is a self-contained component that communicates with others through topics, services, and actions.

## Repository Structure

Each node follows this layout:

```text
<node_name>/
├── peppy.json5          # Node manifest: interfaces, dependencies, variants
└── variants/
    └── <variant_name>/
        ├── peppy.json5  # Variant execution config (build/run commands, container, parameters)
        ├── apptainer.def  # Container definition (if containerized)
        └── src/         # Source code
```

- The root `peppy.json5` declares the node's **interfaces** (topics, services, actions it emits or consumes), its **dependencies** on other nodes, and the list of available **variants**.
- Each variant's `peppy.json5` defines how to **build and run** that specific implementation.

## Variants

Variants are alternative implementations of the same node interface. Only one variant is active at a time. Common variant patterns include:

- **Platform-specific** (e.g. `linux`, `macos`) — real implementations targeting a specific OS
- **Mock** (e.g. `mock_python`, `mock_rust`) — simulated implementations for development and testing
- **Language-specific** (e.g. `rust`, `python`) — same behavior, different language

See the [Peppy documentation](https://github.com/Peppy-bot/peppy) for details on launcher configuration and variant selection.
