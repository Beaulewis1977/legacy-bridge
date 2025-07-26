legacy-bridge-techstack.md

Core Runtime Environments
Node.js: 20 (Alpine-based in Docker)
Rust: 1.75 (stable toolchain)
Python: 3.9 (CI environment)
TypeScript: 5.6.0
Frontend Framework & Libraries
Next.js: 15.4.3
React: 18.3.0
React DOM: 18.3.0
UI Components & Styling
Radix UI Components:
Alert Dialog: 1.1.14
Dropdown Menu: 2.1.15
Label: 2.1.7
Progress: 1.1.7
Separator: 1.1.7
Slot: 1.2.3
Switch: 1.2.5
Tabs: 1.1.12
Tooltip: 1.2.7
Tailwind CSS: 3.4.0
Class Variance Authority: 0.7.1
Tailwind Merge: 3.3.1
Tailwind Variants: 1.0.0
Next Themes: 0.4.6
Framer Motion: 12.23.9
Lucide React: 0.525.0
Data Visualization & Charts
Chart.js: 4.5.0
React ChartJS 2: 5.3.0
Recharts: 3.1.0
D3: 7.9.0
Desktop Application Framework
Tauri: 2.7.0 (API), 2.0.0 (CLI)
Rust Dependencies
Core Libraries
Serde: 1.0 (with derive features)
Serde JSON: 1.0
Regex: 1.10
Thiserror: 1.0
Chrono: 0.4
Pulldown CMark: 0.9
Tokio: 1.42 (full features), 1.35 (multi-thread runtime)
Lazy Static: 1.4
Once Cell: 1.20/1.19
Performance & Concurrency
Rayon: 1.8
AHash: 0.8
SmallVec: 1.11
Bumpalo: 3.14
Crossbeam Channel: 0.5
Crossbeam Deque: 0.8
Num CPUs: 1.16
Parking Lot: 0.12
System & Monitoring
Prometheus: 0.13
Sysinfo: 0.31
Libc: 0.2
Testing & Development Tools
Frontend Testing
Jest: 29.7.0
Testing Library:
Jest DOM: 6.6.0
React: 15.0.0
User Event: 14.6.0
Playwright: 1.49.0
Axe Core Playwright: 4.10.0
Jest Axe: 10.0.0
Rust Testing
Criterion: 0.5 (with HTML reports)
Proptest: 1.4
Pretty Assertions: 1.4
Test Case: 3.3
RSTest: 0.24
Fake: 3.0
Quickcheck: 1.0
Build Tools & Linting
ESLint: 8.57.0
ESLint Config Next: 15.4.3
Concurrently: 8.2.2
TS Jest: 29.2.5
Utility Libraries
Clsx: 2.1.1
Date FNS: 4.1.0
DOMPurify: 3.2.6
File Saver: 2.0.5
JSZip: 3.10.1
WebSocket (ws): 8.18.3
Zustand: 5.0.6
Base64: 0.21
Axios: 1.7.9
Container & Infrastructure
Docker Images
Base Images:
Rust: 1.75-slim, 1.75-alpine
Node.js: 20-alpine
Debian: bookworm-slim
Infrastructure Services:
PostgreSQL: 15-alpine
Redis: 7-alpine
Prometheus: 2.47.0
Grafana: 10.1.0
Jaeger: latest
MinIO: latest
Nginx: alpine
pgAdmin4: latest
Kubernetes & Helm
Helm Chart API: v2
PostgreSQL Chart: 12.x.x (Bitnami)
Redis Chart: 17.x.x (Bitnami)
CI/CD & GitHub Actions
GitHub Actions:
Checkout: v4
Actions RS Toolchain: v1
Setup Python: v4
Docker Compose: 3.9
Development Environment
TypeScript Target: ES2017
Rust Edition: 2021
Security Scanner: Trivy (latest)
This represents a modern, full-stack application with comprehensive testing, monitoring, and deployment infrastructure.