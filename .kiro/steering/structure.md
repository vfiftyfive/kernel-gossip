# Project Structure & Organization

## Root Directory Layout
```
kernel-gossip/
├── crates/                    # Rust workspace crates
├── k8s/                      # Kubernetes manifests
├── pxl-scripts/              # Pixie eBPF scripts
├── docs/                     # User documentation
├── scripts/                  # Utility scripts
├── tests/                    # Integration test suites
├── .kiro/steering/           # AI assistant steering rules
└── .claud-code/              # Development rules and quality gates
```

## Crate Organization
Each crate follows standard Rust project structure with strict separation of concerns:

### `crates/kernel-gossip-types/`
- **Purpose**: Shared type definitions and CRD schemas
- **Key Files**: `kernel_whisper.rs`, `pod_birth_certificate.rs`
- **Pattern**: Pure data types with serde serialization, no business logic

### `crates/kernel-gossip-operator/`
- **Purpose**: Main operator implementation
- **Structure**:
  - `src/main.rs`: Entry point with concurrent server startup
  - `src/config.rs`: Environment-based configuration
  - `src/server.rs`: Axum webhook and metrics servers
  - `src/webhook/`: Webhook handlers and payload processing
  - `src/crd/`: CRD controllers and reconciliation logic
  - `src/actions/`: CRD creation and management actions
  - `src/recommendation/`: Insight generation engine
- **Pattern**: Modular architecture with clear separation between web layer, business logic, and Kubernetes integration

### `crates/kernel-gossip-e2e/`
- **Purpose**: End-to-end integration tests
- **Structure**: Real Kubernetes cluster tests for all scenarios
- **Pattern**: No mocking - tests against actual GKE cluster and Pixie

## Kubernetes Manifests (`k8s/`)
```
k8s/
├── crds/                     # Custom Resource Definitions
├── operator/                 # Operator deployment manifests
├── test-workloads/          # Demo and test workloads
└── namespace.yaml           # Namespace definition
```

## Pixie Scripts (`pxl-scripts/`)
```
pxl-scripts/
├── src/                     # PxL script implementations
│   ├── cpu_throttle_detector.pxl
│   ├── memory_pressure_monitor.pxl
│   ├── network_issue_finder.pxl
│   └── pod_creation_trace.pxl
└── tests/                   # Python validation tests
```

## Documentation Structure
- `README.md`: Project overview and quick start
- `CLAUDE.md`: Master progress tracker and development context
- `docs/DEMO.md`: Demo scenarios and presentation guide
- `docs/PIXIE_INTEGRATION.md`: Pixie setup and integration details
- Individual `claude.md` files in each directory for context preservation

## File Naming Conventions
- **Rust files**: Snake_case (e.g., `kernel_whisper.rs`)
- **Kubernetes manifests**: Kebab-case (e.g., `kernel-whisper.yaml`)
- **Scripts**: Kebab-case with extension (e.g., `build-and-push.sh`)
- **Documentation**: UPPERCASE for root-level (e.g., `README.md`), lowercase for subdirectories

## Development Workflow Patterns
- **TDD First**: All implementation follows strict test-driven development
- **Context Preservation**: Each directory has `claude.md` for AI assistant context
- **Quality Gates**: Automated checks before any commit
- **Real Integration**: No mocking in tests - use actual Kubernetes and Pixie APIs
- **Modular Design**: Clear separation between types, business logic, and infrastructure