# System Architecture Overview

## High-Level Architecture

LegacyBridge is designed as a modular, high-performance document conversion system that bridges legacy RTF systems with modern Markdown workflows.

```mermaid
graph TB
    subgraph "Frontend Layer"
        UI[Next.js UI]
        CLI[CLI Interface]
        API[REST API]
    end
    
    subgraph "Application Layer"
        TAURI[Tauri Runtime]
        WEB[Web Server]
        BATCH[Batch Processor]
    end
    
    subgraph "Core Engine"
        RUST[Rust Conversion Engine]
        PARSER[RTF Parser]
        GEN[Markdown Generator]
        VALID[Validation Layer]
    end
    
    subgraph "Export Layer"
        FFI[FFI Export Layer]
        DLL[Windows DLL]
        SO[Linux SO]
        DYLIB[macOS dylib]
    end
    
    subgraph "Integration Points"
        VB6[VB6 Applications]
        VFP[VFP9 Applications]
        NET[.NET Applications]
        PY[Python Scripts]
        JS[JavaScript/Node.js]
    end
    
    UI --> TAURI
    CLI --> RUST
    API --> WEB
    
    TAURI --> RUST
    WEB --> RUST
    BATCH --> RUST
    
    RUST --> PARSER
    RUST --> GEN
    RUST --> VALID
    
    RUST --> FFI
    FFI --> DLL
    FFI --> SO
    FFI --> DYLIB
    
    DLL --> VB6
    DLL --> VFP
    DLL --> NET
    SO --> PY
    DYLIB --> JS
```

## Component Architecture

```mermaid
graph LR
    subgraph "Rust Core Components"
        direction TB
        
        subgraph "Conversion Pipeline"
            INPUT[Input Handler]
            LEXER[RTF Lexer]
            AST[AST Builder]
            TRANS[Transformer]
            OUTPUT[Output Generator]
            
            INPUT --> LEXER
            LEXER --> AST
            AST --> TRANS
            TRANS --> OUTPUT
        end
        
        subgraph "Support Systems"
            CACHE[Template Cache]
            POOL[Thread Pool]
            LOG[Logger]
            METRIC[Metrics]
        end
        
        subgraph "Security Layer"
            SANITIZE[Input Sanitizer]
            VALIDATE[Format Validator]
            LIMIT[Resource Limiter]
        end
    end
    
    INPUT --> SANITIZE
    SANITIZE --> VALIDATE
    TRANS --> CACHE
    RUST[Engine] --> POOL
    RUST --> LOG
    RUST --> METRIC
```

## Data Flow Architecture

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Engine
    participant Parser
    participant Generator
    participant Cache
    
    Client->>API: Convert Request (RTF/MD)
    API->>Engine: Process Document
    
    Engine->>Parser: Parse Input
    Parser->>Parser: Tokenize
    Parser->>Parser: Build AST
    Parser-->>Engine: Document Model
    
    Engine->>Cache: Check Templates
    Cache-->>Engine: Template Data
    
    Engine->>Generator: Generate Output
    Generator->>Generator: Apply Formatting
    Generator->>Generator: Optimize Output
    Generator-->>Engine: Final Document
    
    Engine-->>API: Conversion Result
    API-->>Client: Response
```

## Memory Management Architecture

```mermaid
graph TD
    subgraph "Memory Allocation"
        STACK[Stack Allocation<br/>- Small strings<br/>- Temporary data]
        HEAP[Heap Allocation<br/>- Large documents<br/>- Dynamic content]
        ARENA[Arena Allocator<br/>- Batch operations<br/>- Temporary objects]
    end
    
    subgraph "Ownership Model"
        RUST_OWN[Rust Ownership<br/>- Automatic cleanup<br/>- No GC needed]
        FFI_PTR[FFI Pointers<br/>- Manual free required<br/>- Reference counting]
    end
    
    subgraph "Safety Mechanisms"
        BOUNDS[Bounds Checking]
        LIFETIME[Lifetime Tracking]
        SANITIZE[Input Sanitization]
    end
    
    STACK --> RUST_OWN
    HEAP --> RUST_OWN
    ARENA --> RUST_OWN
    
    RUST_OWN --> FFI_PTR
    
    BOUNDS --> STACK
    BOUNDS --> HEAP
    LIFETIME --> RUST_OWN
    SANITIZE --> FFI_PTR
```

## Security Architecture

```mermaid
graph TB
    subgraph "Input Layer"
        INPUT[User Input]
        FILE[File Input]
        API_IN[API Input]
    end
    
    subgraph "Validation Layer"
        SIZE[Size Validation<br/>Max 10MB]
        FORMAT[Format Validation<br/>RTF/MD Structure]
        ENCODING[Encoding Check<br/>UTF-8 Validation]
    end
    
    subgraph "Sanitization Layer"
        ESCAPE[HTML Escaping]
        SCRIPT[Script Removal]
        INJECT[Injection Prevention]
    end
    
    subgraph "Processing Layer"
        SANDBOX[Sandboxed Execution]
        TIMEOUT[Timeout Protection]
        RESOURCE[Resource Limits]
    end
    
    subgraph "Output Layer"
        CLEAN[Clean Output]
        AUDIT[Audit Log]
    end
    
    INPUT --> SIZE
    FILE --> SIZE
    API_IN --> SIZE
    
    SIZE --> FORMAT
    FORMAT --> ENCODING
    
    ENCODING --> ESCAPE
    ESCAPE --> SCRIPT
    SCRIPT --> INJECT
    
    INJECT --> SANDBOX
    SANDBOX --> TIMEOUT
    TIMEOUT --> RESOURCE
    
    RESOURCE --> CLEAN
    RESOURCE --> AUDIT
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[HAProxy/Nginx]
    end
    
    subgraph "Application Tier"
        APP1[App Server 1]
        APP2[App Server 2]
        APP3[App Server 3]
    end
    
    subgraph "Cache Layer"
        REDIS1[Redis Primary]
        REDIS2[Redis Replica]
    end
    
    subgraph "Database Tier"
        PG1[PostgreSQL Primary]
        PG2[PostgreSQL Standby]
    end
    
    subgraph "Monitoring"
        PROM[Prometheus]
        GRAF[Grafana]
        ALERT[AlertManager]
    end
    
    LB --> APP1
    LB --> APP2
    LB --> APP3
    
    APP1 --> REDIS1
    APP2 --> REDIS1
    APP3 --> REDIS1
    
    REDIS1 -.-> REDIS2
    
    APP1 --> PG1
    APP2 --> PG1
    APP3 --> PG1
    
    PG1 -.-> PG2
    
    APP1 --> PROM
    APP2 --> PROM
    APP3 --> PROM
    
    PROM --> GRAF
    PROM --> ALERT
```

## Performance Optimization Architecture

```mermaid
graph LR
    subgraph "Optimization Layers"
        direction TB
        
        subgraph "Parse Optimization"
            SIMD[SIMD Instructions]
            PARALLEL[Parallel Tokenization]
            CACHE_PARSE[Parse Cache]
        end
        
        subgraph "Memory Optimization"
            POOL_ALLOC[Object Pooling]
            ARENA_ALLOC[Arena Allocation]
            ZERO_COPY[Zero-Copy Strings]
        end
        
        subgraph "I/O Optimization"
            ASYNC[Async I/O]
            BUFFER[Buffer Management]
            STREAM[Stream Processing]
        end
    end
    
    INPUT[Document Input] --> SIMD
    SIMD --> PARALLEL
    PARALLEL --> CACHE_PARSE
    
    CACHE_PARSE --> POOL_ALLOC
    POOL_ALLOC --> ARENA_ALLOC
    ARENA_ALLOC --> ZERO_COPY
    
    ZERO_COPY --> ASYNC
    ASYNC --> BUFFER
    BUFFER --> STREAM
    
    STREAM --> OUTPUT[Optimized Output]
```

## Error Handling Architecture

```mermaid
stateDiagram-v2
    [*] --> Input
    Input --> Validation
    
    Validation --> Processing: Valid
    Validation --> ErrorHandler: Invalid
    
    Processing --> Conversion
    Processing --> ErrorHandler: ProcessError
    
    Conversion --> Output: Success
    Conversion --> ErrorRecovery: ConversionError
    
    ErrorRecovery --> PartialOutput: Recoverable
    ErrorRecovery --> ErrorHandler: Unrecoverable
    
    ErrorHandler --> ErrorLog
    ErrorHandler --> UserNotification
    
    ErrorLog --> [*]
    UserNotification --> [*]
    Output --> [*]
    PartialOutput --> [*]
```

## Scalability Architecture

```mermaid
graph TB
    subgraph "Horizontal Scaling"
        direction LR
        
        subgraph "Worker Pool"
            W1[Worker 1]
            W2[Worker 2]
            W3[Worker 3]
            WN[Worker N]
        end
        
        subgraph "Queue System"
            Q1[Priority Queue]
            Q2[Batch Queue]
            Q3[Background Queue]
        end
    end
    
    subgraph "Vertical Scaling"
        direction TB
        
        subgraph "Resource Allocation"
            CPU[CPU Cores<br/>Auto-scaling]
            MEM[Memory<br/>Dynamic allocation]
            DISK[Disk I/O<br/>SSD optimization]
        end
    end
    
    subgraph "Load Distribution"
        BALANCE[Load Balancer]
        SHARD[Document Sharding]
        PART[Partition Strategy]
    end
    
    BALANCE --> W1
    BALANCE --> W2
    BALANCE --> W3
    BALANCE --> WN
    
    W1 --> Q1
    W2 --> Q2
    W3 --> Q3
    
    Q1 --> CPU
    Q2 --> MEM
    Q3 --> DISK
    
    CPU --> SHARD
    MEM --> SHARD
    DISK --> PART
```

## Platform Integration Architecture

```mermaid
graph TD
    subgraph "Legacy Platforms"
        VB6APP[VB6 Application]
        VFPAPP[VFP9 Application]
        DELPHI[Delphi Application]
    end
    
    subgraph "Modern Platforms"
        NETCORE[.NET Core/5+]
        NODEJS[Node.js]
        PYTHON[Python]
        JAVA[Java]
    end
    
    subgraph "LegacyBridge Core"
        DLL32[32-bit DLL]
        DLL64[64-bit DLL]
        LINUX[Linux SO]
        MAC[macOS dylib]
    end
    
    subgraph "Wrapper Layer"
        VBWRAP[VB6 Wrapper]
        VFPWRAP[VFP9 Wrapper]
        NETWRAP[.NET Wrapper]
        PYWRAP[Python Wrapper]
    end
    
    VB6APP --> VBWRAP
    VFPAPP --> VFPWRAP
    DELPHI --> DLL32
    
    NETCORE --> NETWRAP
    NODEJS --> DLL64
    PYTHON --> PYWRAP
    JAVA --> LINUX
    
    VBWRAP --> DLL32
    VFPWRAP --> DLL32
    NETWRAP --> DLL64
    PYWRAP --> LINUX
```

## Key Architecture Principles

### 1. **Modularity**
- Clear separation of concerns
- Pluggable components
- Independent scaling

### 2. **Performance**
- Zero-copy operations where possible
- SIMD optimization for parsing
- Efficient memory management

### 3. **Security**
- Input validation at every layer
- Sandboxed execution
- Resource limits enforced

### 4. **Compatibility**
- Multiple platform support
- Legacy system integration
- Forward compatibility

### 5. **Reliability**
- Comprehensive error handling
- Graceful degradation
- Automatic recovery

## Architecture Decision Records (ADRs)

### ADR-001: Rust as Core Language
- **Decision**: Use Rust for the core conversion engine
- **Rationale**: Memory safety, performance, and FFI capabilities
- **Consequences**: Excellent performance and safety, steeper learning curve

### ADR-002: FFI for Legacy Integration
- **Decision**: Use C-compatible FFI for legacy system integration
- **Rationale**: Maximum compatibility with VB6, VFP9, and other legacy systems
- **Consequences**: Manual memory management at boundaries, excellent compatibility

### ADR-003: Template-Based RTF Generation
- **Decision**: Use template system for RTF output
- **Rationale**: Consistent formatting, easier customization
- **Consequences**: Better maintainability, slight performance overhead

### ADR-004: 10MB File Size Limit
- **Decision**: Impose 10MB limit on single document conversions
- **Rationale**: Prevent DoS attacks, ensure predictable performance
- **Consequences**: Large documents need splitting, protects system resources

---

For interactive architecture diagrams, see [Component Diagrams](component-diagrams/).