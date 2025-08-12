# ADR-001: Model Context Protocol (MCP) Server Implementation for FHIRPath

## Status

Proposed

## Context

The OctoFHIR FHIRPath Rust library has evolved into a comprehensive, high-performance implementation of the FHIRPath specification with a modular workspace architecture. To extend its utility and make it accessible to AI assistants and other tools, we need to provide a standardized interface that allows external systems to leverage FHIRPath capabilities.

The Model Context Protocol (MCP) is an open standard created by Anthropic that enables AI assistants to securely connect to data sources and tools. MCP provides a standardized way to expose functionality through:

- **Tools**: Functions that can be called by LLMs (with user approval)
- **Resources**: File-like data that can be read by clients
- **Prompts**: Pre-written templates for common tasks

### Current State

Our FHIRPath library provides:
- **High-performance FHIRPath evaluation**: 1M+ operations/second parser, arena-based memory management
- **Comprehensive FHIR support**: 82.7% specification compliance (831/1005 official tests pass)
- **Modular architecture**: 11 specialized crates (ast, parser, model, evaluator, registry, compiler, diagnostics, etc.)
- **Advanced features**: Bytecode compilation, VM execution, extension system, caching
- **Clean API**: Well-designed public interface through `octofhir-fhirpath` crate

### Problem Statement

Currently, users must:
1. Have Rust development environment installed
2. Write Rust code to integrate the library
3. Manually handle FHIR resource loading and validation
4. Understand complex FHIRPath syntax and FHIR data structures

This creates barriers for:
- AI assistants that need FHIRPath evaluation capabilities
- Non-Rust developers wanting to use FHIRPath
- Interactive exploration of FHIR data
- Integration with existing healthcare tools and workflows

### Market Opportunity

**First-Mover Advantage**: No existing MCP server provides comprehensive FHIRPath capabilities. By implementing a best-in-class solution with both stdio and HTTP/SSE transports from the start, we can establish the standard for FHIR data interaction in the AI ecosystem.

## Decision

We will implement a Model Context Protocol (MCP) server crate (`fhirpath-mcp-server`) that exposes core FHIRPath functionality through standardized MCP interfaces, enabling AI assistants and other MCP clients to:

1. **Evaluate FHIRPath expressions** against FHIR resources
2. **Parse and validate FHIRPath expressions** for syntax correctness
3. **Extract specific data** from FHIR resources using FHIRPath queries
4. **Access FHIR schemas and examples** for context and learning

### Architecture Design

#### 1. Best-in-Class Crate Structure
```
crates/fhirpath-mcp-server/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API for library usage
│   ├── server.rs           # Core MCP server implementation
│   ├── transport/          # Transport layer implementations
│   │   ├── mod.rs
│   │   ├── stdio.rs        # Standard I/O transport
│   │   ├── http.rs         # HTTP/SSE transport
│   │   └── websocket.rs    # WebSocket transport (future)
│   ├── tools/              # Core MCP tools implementation
│   │   ├── mod.rs
│   │   ├── evaluate.rs     # FHIRPath evaluation with caching
│   │   ├── parse.rs        # Expression parsing and validation
│   │   ├── extract.rs      # Data extraction using FHIRPath
│   │   └── explain.rs      # Expression explanation using fhirpath-analyzer
│   ├── resources/          # MCP resources implementation
│   │   ├── mod.rs
│   │   ├── schemas.rs      # FHIR schema resources (R4/R5)
│   │   ├── examples.rs     # Curated FHIR examples
│   │   └── documentation.rs # FHIRPath syntax and function docs
│   ├── prompts/            # MCP prompts implementation
│   │   ├── mod.rs
│   │   ├── common.rs       # Common FHIRPath patterns
│   │   ├── extraction.rs   # Data extraction guides
│   │   └── learning.rs     # FHIRPath learning templates
│   ├── security/           # Security and authentication
│   │   ├── mod.rs
│   │   ├── auth.rs         # Authentication mechanisms
│   │   ├── rate_limit.rs   # Rate limiting
│   │   └── cors.rs         # CORS handling
│   ├── cache/              # Performance optimization
│   │   ├── mod.rs
│   │   ├── expression.rs   # Expression compilation cache
│   │   ├── resource.rs     # Resource validation cache
│   │   └── memory.rs       # Memory management
│   ├── metrics/            # Observability
│   │   ├── mod.rs
│   │   ├── telemetry.rs    # OpenTelemetry integration
│   │   └── health.rs       # Health check endpoints
│   ├── bin/
│   │   ├── fhirpath-mcp.rs     # Standalone server binary
│   │   ├── benchmark.rs        # Performance benchmarking
│   │   └── validate-server.rs  # Server validation tool
│   ├── config/             # Advanced configuration
│   │   ├── mod.rs
│   │   ├── server.rs       # Server configuration
│   │   ├── security.rs     # Security settings
│   │   └── performance.rs  # Performance tuning
│   └── middleware/         # HTTP middleware stack
│       ├── mod.rs
│       ├── logging.rs      # Request logging
│       ├── compression.rs  # Response compression
│       └── timeout.rs      # Request timeout handling
```

#### 2. Core MCP Tools Implementation

**Tool: `evaluate_fhirpath`**
- **Purpose**: High-performance FHIRPath evaluation with intelligent caching
- **Parameters**:
  - `expression`: FHIRPath expression string
  - `resource`: FHIR resource (JSON)
  - `context` (optional): Additional context variables
  - `timeout_ms` (optional): Evaluation timeout in milliseconds
- **Returns**: Evaluation results with type information and performance metrics
- **Features**: Expression compilation caching, optimized evaluation engine

**Tool: `parse_fhirpath`**
- **Purpose**: Parse and validate FHIRPath expressions with detailed feedback
- **Parameters**:
  - `expression`: FHIRPath expression string
  - `include_ast` (optional): Include AST representation in response
  - `explain_syntax` (optional): Provide syntax explanation
- **Returns**: Parsing results, syntax validation, AST structure, error details
- **Features**: Comprehensive syntax validation, educational explanations

**Tool: `extract_data`**
- **Purpose**: Extract specific data from FHIR resources using FHIRPath
- **Parameters**:
  - `resource`: FHIR resource (JSON)
  - `expressions`: Array of FHIRPath expressions with labels
  - `flatten_results` (optional): Flatten nested extraction results
- **Returns**: Extracted values mapped to expression labels
- **Features**: Multiple concurrent extractions, structured output

**Tool: `explain_expression`**
- **Purpose**: Provide detailed explanation of FHIRPath expressions
- **Parameters**:
  - `expression`: FHIRPath expression string
  - `context_type` (optional): FHIR resource type for context-specific help
- **Returns**: Step-by-step explanation, function documentation, examples
- **Features**: Educational support, context-aware explanations

#### 3. MCP Resources Implementation

**Resource: `fhir_schemas`**
- **Purpose**: Provide access to FHIR schema definitions for context
- **Content**: FHIR R4/R5 structure definitions and element paths
- **URI pattern**: `fhir://schema/{resource_type}`

**Resource: `example_resources`**
- **Purpose**: Provide example FHIR resources for testing FHIRPath expressions
- **Content**: Curated set of valid FHIR resources with common data patterns
- **URI pattern**: `fhir://examples/{resource_type}/{example_name}`

**Resource: `fhirpath_documentation`**
- **Purpose**: Provide FHIRPath syntax and function documentation
- **Content**: Function reference, syntax guides, common patterns
- **URI pattern**: `fhirpath://docs/{topic}`

#### 4. MCP Prompts Implementation

**Prompt: `common_patterns`**
- **Purpose**: Help users with common FHIRPath extraction patterns
- **Examples**: Patient demographics, observation values, medication details, care team members

**Prompt: `learning_fhirpath`**
- **Purpose**: Guide users through FHIRPath learning and practice
- **Examples**: Basic syntax, function usage, expression building, debugging techniques

**Prompt: `expression_debugging`**
- **Purpose**: Help users debug and optimize FHIRPath expressions
- **Examples**: Common errors, performance tips, alternative approaches

#### 5. Distribution Strategy

##### Cross-Platform Binaries via GitHub Releases
```
Release Artifacts:
├── fhirpath-mcp-{version}-x86_64-unknown-linux-gnu
├── fhirpath-mcp-{version}-x86_64-pc-windows-msvc.exe
├── fhirpath-mcp-{version}-x86_64-apple-darwin
├── fhirpath-mcp-{version}-aarch64-apple-darwin
└── checksums.txt
```

**GitHub Actions Workflow**:
- Cross-compilation for major platforms (Linux, Windows, macOS)
- Automated testing on all target platforms
- Release artifact generation with checksums
- Docker image building and publishing

##### Docker Images
```dockerfile
# Multi-stage build for minimal runtime image
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin fhirpath-mcp

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fhirpath-mcp /usr/local/bin/
ENTRYPOINT ["fhirpath-mcp"]
```

**Docker Registry Strategy**:
- Primary: GitHub Container Registry (`ghcr.io/octofhir/fhirpath-mcp`)
- Secondary: Docker Hub (`octofhir/fhirpath-mcp`)
- Multi-architecture support (amd64, arm64)
- Semantic versioning with `latest` tag

#### 6. Integration Points

**With Existing Workspace**:
- Depends on `octofhir-fhirpath` for core functionality
- Uses `fhirpath-diagnostics` for error reporting
- Leverages `fhirpath-analyzer` for expression explanation (see ADR-002)
- Integrates with `fhirpath-tools` for validation utilities

**MCP SDK Integration**:
- Uses official Rust MCP SDK (`rmcp`) with custom extensions
- Tokio async runtime for maximum performance
- **Multi-transport support from Day 1**:
  - Stdio transport for local CLI integration
  - HTTP/SSE transport for web applications and remote access
  - Future: WebSocket transport for real-time applications

**Security Architecture**:
- **Authentication**: JWT tokens, API keys, OAuth 2.0/OIDC support
- **Authorization**: Role-based access control (RBAC) for different tool sets
- **Rate Limiting**: Adaptive rate limiting with user-specific quotas
- **Input Validation**: Comprehensive FHIRPath expression sanitization
- **CORS Support**: Configurable cross-origin resource sharing
- **TLS/SSL**: Mandatory encryption for HTTP transport
- **Audit Logging**: Complete request/response audit trail

**Performance Architecture**:
- **Intelligent Caching**: Multi-level caching (expression compilation, resource validation, schema)
- **Connection Pooling**: Efficient connection management for HTTP transport
- **Streaming Support**: Large resource processing without memory exhaustion
- **Compression**: Automatic response compression (gzip, brotli)
- **Request Timeout**: Configurable timeouts with graceful degradation
- **Circuit Breaker**: Fault tolerance for external resource dependencies
- **Memory Management**: Arena-based allocation with automatic cleanup

**Observability Architecture**:
- **OpenTelemetry Integration**: Distributed tracing and metrics
- **Health Checks**: Comprehensive health and readiness endpoints
- **Performance Metrics**: Real-time performance monitoring
- **Error Tracking**: Structured error reporting with context
- **Request Analytics**: Usage patterns and optimization insights

### Implementation Phases (First-Mover Strategy)

#### Phase 1: Core FHIRPath MCP Server (Weeks 1-2)
- Set up `fhirpath-mcp-server` crate with modular architecture
- Implement core MCP server with both **stdio and HTTP/SSE transports**
- Implement `evaluate_fhirpath` and `parse_fhirpath` tools
- Basic security (authentication, input validation)
- Performance monitoring and health checks
- **Goal**: First working FHIRPath MCP server with dual transport support

#### Phase 2: Essential Tooling (Weeks 3-4)
- Implement `extract_data` and `explain_expression` tools
- Intelligent expression compilation caching
- Comprehensive error handling and diagnostics
- Basic FHIR schema resources for context
- **Goal**: Complete core FHIRPath functionality via MCP

#### Phase 3: Production Features (Weeks 5-6)
- Security enhancements: rate limiting, audit logging
- Example FHIR resources and FHIRPath documentation resources
- Prompt library for common patterns and learning
- OpenTelemetry integration for observability
- **Goal**: Production-ready FHIRPath MCP server

#### Phase 4: Distribution and Integration (Weeks 7-8)
- Cross-platform distribution (GitHub releases + Docker)
- Integration testing with Claude Code and other MCP clients
- Performance benchmarking and optimization
- Community documentation and usage examples
- **Goal**: Establish market presence and developer adoption

#### Phase 5: Ecosystem Growth (Weeks 9-10)
- WebSocket transport for real-time applications
- Advanced caching and performance optimization
- Community feedback integration and enhancements
- Usage analytics and monitoring improvements
- **Goal**: Solidify position as the standard FHIRPath MCP solution

## Consequences

### Positive
- **First-Mover Advantage**: Establishes the standard for FHIRPath in AI ecosystems
- **Market Leadership**: Potential to become the de facto FHIRPath MCP implementation
- **Broader Accessibility**: Makes FHIRPath available to AI assistants and non-Rust developers
- **Enterprise Ready**: Production-grade security, performance, and observability from the start
- **Comprehensive Feature Set**: Most complete FHIRPath tooling available through MCP
- **Multi-Transport Support**: Flexible deployment options for any environment
- **Performance Leadership**: Leverages our high-performance Rust implementation
- **Ecosystem Growth**: Drives adoption of our FHIRPath library across the industry

### Negative
- **Increased Scope**: More complex initial implementation requiring significant effort
- **Resource Requirements**: Enterprise features require more development and maintenance resources
- **Security Surface**: HTTP transport and authentication increase security considerations
- **Market Responsibility**: As first-mover, we set expectations for the entire ecosystem

### Strategic Considerations
- **Competition Timeline**: Need to move fast to maintain first-mover advantage
- **Quality vs Speed**: Must balance rapid development with production-quality implementation
- **Community Building**: Success depends on developer adoption and community engagement
- **Long-term Commitment**: Establishing a standard requires ongoing innovation and support

## Implementation Checklist

### Phase 1: Core Foundation (Weeks 1-2)
- [ ] Create `fhirpath-mcp-server` crate with modular architecture
- [ ] Implement MCP server core with stdio and HTTP/SSE transports
- [ ] Develop `evaluate_fhirpath` and `parse_fhirpath` tools
- [ ] Implement basic authentication and input validation
- [ ] Add health checks and performance monitoring
- [ ] Create standalone binary with configuration management
- [ ] Basic integration testing with MCP clients

### Phase 2: Essential Tools (Weeks 3-4)
- [ ] Implement `extract_data` tool with multi-expression support
- [ ] Implement `explain_expression` tool using fhirpath-analyzer crate
- [ ] Build expression compilation caching system
- [ ] Add comprehensive error handling and diagnostics
- [ ] Implement basic FHIR schema resources
- [ ] Performance optimization for core operations
- [ ] Extended integration testing

### Phase 3: Production Ready (Weeks 5-6)
- [ ] Enhanced security: rate limiting, audit logging
- [ ] Example FHIR resources and documentation resources
- [ ] Prompt library for common patterns and learning
- [ ] OpenTelemetry integration and metrics
- [ ] Production configuration management
- [ ] Security review and testing

### Phase 4: Distribution (Weeks 7-8)
- [ ] Cross-platform GitHub Actions build pipeline
- [ ] Docker images with multi-architecture support
- [ ] Performance benchmarking and optimization
- [ ] Integration with Claude Code and other MCP clients
- [ ] Community documentation and examples
- [ ] Public release and announcement

### Phase 5: Ecosystem Growth (Weeks 9-10)
- [ ] WebSocket transport implementation
- [ ] Advanced caching and performance features
- [ ] Community feedback integration
- [ ] Usage analytics and monitoring
- [ ] Continuous optimization and enhancement

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [FHIRPath Specification](http://hl7.org/fhirpath/)
- [ADR-002: FHIRPath Analyzer Crate](./002-fhirpath-analyzer-crate.md)
- [Architecture Decision Record Template](https://github.com/joelparkerhenderson/architecture-decision-record)