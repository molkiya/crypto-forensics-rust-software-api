# Bitcoin AML Detection using Graph Neural Networks Rust API app

Bitcoin transaction analysis system with web interface and LLM integration for detecting suspicious activity.

> 🇬🇧 [English version](README_EN.md) | 🇷🇺 [Русская версия](README.md)

## Description

This project is a Rust web application for analyzing Bitcoin transactions and addresses. The system extracts and analyzes various transaction characteristics, visualizes transaction graphs, and integrates with a Python repository that uses LLM for transaction classification.

## Key Features

- **Transaction Analysis**: Extraction and analysis of Bitcoin transaction characteristics
- **Address Analysis**: Retrieval of information about Bitcoin addresses and their activity
- **Graph Visualization**: Building transaction graphs with color coding
- **Web Interface**: Intuitive web interface for working with the system
- **LLM Integration**: Integration with Python repository for ML analysis of transactions

## Project Structure

```
diploma_software/
├── src/
│   ├── application/          # Application business logic
│   │   ├── services/          # Application services
│   │   │   └── transaction/  # Transaction services
│   │   └── plot_render.rs    # Graph rendering
│   ├── infrastructure/       # Infrastructure layer
│   │   ├── constants.rs      # Application constants
│   │   ├── config.rs         # Configuration management
│   │   └── model/            # ML models
│   ├── services/             # External services
│   │   └── explorer/         # Bitcoin explorer client
│   ├── common/               # Common utilities
│   ├── utils/                # Helper functions
│   ├── data/                 # Analysis data
│   ├── lib.rs                # Library entry point
│   ├── main.rs               # CLI entry point
│   └── main-api.rs           # Web server entry point
├── static/                   # Static files (HTML templates)
├── Cargo.toml                # Project dependencies
└── LICENSE                   # Apache 2.0 License
```

## Requirements

- Rust 1.70+ (edition 2021)
- Cargo
- Access to Bitcoin explorer API (configured via environment variables)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd diploma_software
```

2. Create a `.env` file in the project root (see `.env.example` if available):
```bash
# Example configuration
BITCOIN_EXPLORER_URL=<your-explorer-url>
TEST_TX_ID=<optional-test-transaction-id>
DATA_DIR=./src/data
DEFAULT_DATA_FOLDER=1111DAYXhoxZx2tsRnzimfozo783x1yC2
```

3. Build the project:
```bash
cargo build --release
```

## Usage

### Running the Web Server

```bash
cargo run --bin main-api
```

The web server will automatically find an available port in the specified range and open a browser with the main page.

### Running the CLI Application

```bash
cargo run --bin main
```

## API Endpoints

- `GET /` - Main page
- `POST /confirm` - Analyze data by folder
- `GET /tx/{tx_id}` - Transaction information
- `GET /address/{address}` - Address information

## Integration with Python LLM

The project is designed to work with a Python repository that uses LLM for transaction analysis. Integration is done through:

- Data exchange in JSON format
- Using shared ML models (e.g., `aml_bitcoin.pth`)
- API for transmitting transaction data to Python service

### Data Model for Python Service

The Rust application sends the following data to the Python service:

**Minimum set (current implementation):**
- `transaction_id` - Transaction ID
- `transaction_features` - Basic characteristics:
  - `n_inputs`, `n_outputs` - Number of inputs/outputs
  - `input_value_sum`, `output_value_sum` - Amounts in BTC
  - `transaction_fee` - Transaction fee
  - `avg_input_value`, `avg_output_value` - Average values

**Full set (planned):**
- All of the above +
- `extended_features` - 15 additional features
- `addresses` - Lists of input and output addresses
- `input_features` / `output_features` - Features for each address (55 features)

**Expected response from Python:**
```json
{
  "success": true,
  "prediction": {
    "class": "illicit" | "licit" | "unknown",
    "confidence": 0.0-1.0,
    "risk_score": 0.0-1.0
  },
  "explanation": "Text explanation"
}
```

📖 **Full API specification**: see [PYTHON_API_SPEC.md](PYTHON_API_SPEC.md) and [INTEGRATION.md](INTEGRATION.md)

## Development

### Dependencies

Main project dependencies:
- `actix-web` - Web framework
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `bitcoin` - Bitcoin support
- `blockbook` - Bitcoin explorer client
- `tera` - Template engine
- `plotly` - Data visualization

### Code Structure

The project follows clean architecture principles:
- **Application** - Business logic
- **Infrastructure** - Infrastructure components
- **Services** - External services and integrations

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_transaction_features_calculation
```

### Code Quality

The project includes:
- ✅ Proper error handling (no `unwrap()` in production code)
- ✅ Unit tests for core functionality
- ✅ Documentation comments for public APIs
- ✅ Configuration management
- ✅ Type-safe error handling with `thiserror`

## Configuration

The application can be configured via environment variables:

- `BITCOIN_EXPLORER_URL` - Bitcoin explorer API URL (default: `https://mempool.space/api`)
- `TEST_TX_ID` - Test transaction ID for demonstration
- `DATA_DIR` - Path to data directory (default: `./src/data`)
- `DEFAULT_DATA_FOLDER` - Default data folder name
- `RUST_LOG` - Logging level (e.g., `debug`, `info`, `warn`)

## License

The project is licensed under Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## Authors

Marat Kiiamov (kiya.marat@gmail.com)

## Acknowledgments

- Bitcoin Core
- Blockbook Explorer
- Rust Community

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

