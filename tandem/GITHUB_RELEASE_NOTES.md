# Tandem v0.6.4

Tandem v0.6.4 is a small consistency release for the retired `ready` accord action.

## CLI and protocol

- Bare `tandem accord` usage now lists only the supported actions: `claim`, `deliver`, `accept`, `rework`, `block`, and `fail`.
- `tandem accord ready` remains rejected with a clear current-action list.
- Existing persisted `accord.status: ready` records remain readable for compatibility.

## Pi-Tandem integration

- Centralized the accepted accord action list across Pi-Tandem’s type, runtime argument builder, and tool schema.
- Added regression coverage ensuring the retired `ready` action is neither advertised nor accepted by the adapter.
- Updated repository guidance and release documentation to describe direct claims consistently.
