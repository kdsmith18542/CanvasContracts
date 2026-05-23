# Canvas Contracts

**Paint Your Logic. Deploy Your Future.**

Canvas Contracts is a visual smart contract development platform that allows developers to build, test, and deploy WebAssembly (WASM) smart contracts using an intuitive drag-and-drop interface. Built on top of the BaaLS (Blockchain as a Local Service) engine, it provides a seamless development experience for both beginners and experienced blockchain developers.

## 🎨 Features

- **Visual Contract Builder**: Drag-and-drop interface for creating smart contracts
- **Multi-Language Support**: Write custom logic in Rust, Go, AssemblyScript, or F#
- **WASM Compilation**: Automatic compilation of visual graphs to optimized WASM modules
- **Local Simulation**: Built-in sandbox for testing contracts before deployment
- **BaaLS Integration**: Seamless integration with the BaaLS blockchain engine
- **Real-time Validation**: AI-powered suggestions and security checks
- **Cross-Platform**: Desktop application built with Tauri

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (v18+)
- **wasm-pack** for WASM compilation
- **wasmtime** for local execution

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/kdsmith18542/CanvasContracts.git
   cd CanvasContracts
   ```

2. **Install all dependencies**
   ```bash
   make install
   ```

3. **Build the application**
   ```bash
   make build-app
   ```

4. **Start the development server**
   ```bash
   make run-app
   ```

## 🏗️ Architecture

Canvas Contracts consists of several key components:

### Frontend (React + Tauri)
- **Visual Editor**: Drag-and-drop interface for building contracts
- **Node Palette**: Library of pre-built contract components
- **Property Panel**: Configuration interface for nodes
- **Debugger**: Real-time execution tracing and state inspection

### Backend (Rust)
- **Contract Compiler**: Converts visual graphs to WASM bytecode
- **Node Definition Language (NDL)**: Schema for defining new node types
- **BaaLS SDK Integration**: Direct integration with the BaaLS engine
- **WASM Runtime**: Local execution environment using wasmtime

### AI Assistant (Non-LLM)
- **Pattern Recognition**: Identifies common contract patterns
- **Security Validation**: Rule-based security checks
- **Optimization Suggestions**: Gas optimization recommendations
- **Test Generation**: Automated test case generation

## 📚 Documentation

- [User Guide](docs/user-guide.md) - How to use Canvas Contracts
- [Developer Guide](docs/developer-guide.md) - Contributing to the project
- [API Reference](docs/api-reference.md) - Technical documentation
- [Architecture](docs/architecture.md) - System design and components

## 🧪 Testing

### Run Tests
```bash
# Rust tests
make test

# Frontend tests
cd frontend && npm test

# Integration tests
npm run test:integration
```

### Code Quality
```bash
# Format code
make fmt

# Lint code
make lint
```

## 🔧 Development

### Development Environment

- **Rust toolchain**: See `rust-toolchain.toml` (run `rustup show` to check)
- **VSCode**: Recommended (see `.vscode/extensions.json`)
- **.env**: Copy `.env.example` to `.env` and fill in values as needed

### Development Commands

#### Backend (Rust)
- `make build` - Build the entire workspace
- `make test` - Run all tests
- `make lint` - Run clippy linter
- `make fmt` - Format code with rustfmt
- `make clean` - Clean build artifacts
- `make run` - Run CLI only

#### Frontend (React + Tauri)
- `make frontend-install` - Install frontend dependencies
- `make frontend-dev` - Start frontend development server
- `make frontend-build` - Build frontend
- `make tauri-dev` - Start Tauri development
- `make tauri-build` - Build Tauri application

#### Full Application
- `make install` - Install all dependencies
- `make build-app` - Build full application
- `make run-app` - Run full Tauri application

### Project Structure
```
canvascontract/
├── src/                    # Rust backend source
│   ├── compiler/          # Contract compilation engine
│   ├── nodes/             # Node definitions and implementations
│   ├── wasm/              # WASM runtime integration
│   └── baals/             # BaaLS SDK integration
├── frontend/              # React frontend application
│   ├── src/
│   │   ├── components/    # React components
│   │   ├── hooks/         # Custom React hooks
│   │   ├── utils/         # Utility functions
│   │   └── types/         # TypeScript type definitions
│   └── public/            # Static assets
├── contracts/             # Example contracts and templates
├── docs/                  # Documentation
└── tests/                 # Test suites
```

### Adding New Node Types

1. **Define the node in NDL** (`src/nodes/definitions/`)
2. **Implement the node logic** (`src/nodes/implementations/`)
3. **Add visual representation** (`frontend/src/components/nodes/`)
4. **Write tests** (`tests/nodes/`)

### Building Custom WASM Modules

```rust
// Example custom node implementation
use canvas_contracts::node::{Node, NodeContext};

pub struct CustomNode {
    // Node properties
}

impl Node for CustomNode {
    fn execute(&self, context: &mut NodeContext) -> Result<(), Box<dyn std::error::Error>> {
        // Custom logic here
        Ok(())
    }
}
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Style

- **Rust**: Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- **TypeScript/JavaScript**: Use ESLint and Prettier configurations
- **Commits**: Use conventional commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **BaaLS Team** - For the foundational blockchain engine
- **WASM Community** - For the WebAssembly ecosystem
- **React Flow** - For the visual programming framework
- **wasmtime** - For the WASM runtime

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/kdsmith18542/CanvasContracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kdsmith18542/CanvasContracts/discussions)
- **Documentation**: [Project Wiki](https://github.com/kdsmith18542/CanvasContracts/wiki)

## 🔗 Links

- **Website**: [canvascontracts.com](https://canvascontracts.com)
- **Documentation**: [docs.canvascontracts.com](https://docs.canvascontracts.com) 
