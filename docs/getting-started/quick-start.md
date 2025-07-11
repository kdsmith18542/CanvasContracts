# Quick Start Guide

Get up and running with Canvas Contracts in minutes! This guide will walk you through installing the platform and creating your first visual smart contract.

## Prerequisites

Before you begin, make sure you have the following installed:

- **Rust** (1.70 or later): [Install Rust](https://rustup.rs/)
- **Node.js** (18 or later): [Install Node.js](https://nodejs.org/)
- **Git**: [Install Git](https://git-scm.com/)

## Installation

### Option 1: Clone and Build

1. **Clone the repository**
   ```bash
   git clone https://github.com/kdsmith18542/CanvasContracts
   cd CanvasContracts
   ```

2. **Install dependencies and build**
   ```bash
   make install
   ```

3. **Verify installation**
   ```bash
   canvas-contracts --version
   ```

### Option 2: Using Cargo

```bash
cargo install canvas-contracts
```

## Starting the Visual Editor

Launch the Canvas Contracts visual editor:

```bash
canvas-contracts editor
```

This will open the Tauri-based desktop application with the visual contract editor.

## Creating Your First Contract

### Step 1: Create a New Project

1. Click **"New Project"** in the welcome screen
2. Select **"Blank Canvas"** template
3. Name your project (e.g., "My First Contract")
4. Click **"Create"**

### Step 2: Add Nodes to the Canvas

1. **Start Node**: Drag a "Start" node from the palette to the canvas
2. **Logic Node**: Add an "If" node for conditional logic
3. **State Node**: Add a "Write Storage" node for data persistence
4. **End Node**: Add an "End" node to complete the flow

### Step 3: Connect the Nodes

1. Click and drag from the output port of one node to the input port of another
2. The system will highlight valid connections
3. Connect your nodes to create the execution flow

### Step 4: Configure Node Properties

1. Select a node to open the properties panel
2. Configure the node's behavior:
   - **If Node**: Set the condition (e.g., "input > 100")
   - **Write Storage Node**: Set the key and value
   - **End Node**: Set the return value

### Step 5: Validate Your Contract

1. Click the **"Validate"** button in the toolbar
2. Review any warnings or errors
3. Fix issues as needed

### Step 6: Test Your Contract

1. Click the **"Test"** button
2. Set input values in the test panel
3. Run the simulation
4. Review the execution trace and results

### Step 7: Deploy Your Contract

1. Click the **"Deploy"** button
2. Choose your deployment target (local BaaLS node)
3. Review the deployment configuration
4. Click **"Deploy"** to publish your contract

## Example: Simple Voting Contract

Let's create a basic voting contract to demonstrate the platform:

### Contract Logic

1. **Start**: Initialize the contract
2. **Get Sender**: Get the caller's address
3. **Check Balance**: Verify the caller has voting tokens
4. **If**: Check if they've already voted
5. **Write Storage**: Record the vote
6. **Emit Event**: Notify about the vote
7. **End**: Return success

### Step-by-Step Creation

1. **Add Start Node**
   - Drag "Start" node to canvas
   - Position at top center

2. **Add Get Sender Node**
   - Drag "Get Sender" node below Start
   - Connect Start output to Get Sender input

3. **Add Check Balance Node**
   - Drag "Read Storage" node
   - Configure to check voting balance
   - Connect Get Sender to Check Balance

4. **Add If Node**
   - Drag "If" node
   - Set condition: "balance > 0 AND !hasVoted"
   - Connect Check Balance to If

5. **Add Write Storage Node (True Path)**
   - Drag "Write Storage" node
   - Configure to record the vote
   - Connect If "true" output to Write Storage

6. **Add Emit Event Node**
   - Drag "Emit Event" node
   - Configure event: "VoteCast"
   - Connect Write Storage to Emit Event

7. **Add End Node**
   - Drag "End" node
   - Connect Emit Event to End

8. **Add Error Path**
   - Drag another "End" node
   - Connect If "false" output to this End
   - Set return value to error message

### Testing the Contract

1. **Set Test Inputs**
   ```json
   {
     "sender": "0x1234...",
     "vote": "yes"
   }
   ```

2. **Run Simulation**
   - Click "Test" button
   - Review execution trace
   - Check storage changes
   - Verify events emitted

3. **Deploy**
   - Click "Deploy" button
   - Choose local BaaLS node
   - Confirm deployment

## Using the AI Assistant

The AI Assistant provides intelligent suggestions:

1. **Pattern Recognition**: Identifies common contract patterns
2. **Security Analysis**: Detects potential vulnerabilities
3. **Optimization**: Suggests gas-saving improvements
4. **Node Recommendations**: Suggests next nodes based on context

### Accessing AI Features

1. Click the **"AI Assistant"** button in the toolbar
2. Review the analysis tabs:
   - **Patterns**: Recognized contract patterns
   - **Issues**: Security and optimization issues
   - **Suggestions**: Recommended improvements

## Next Steps

Now that you've created your first contract, explore:

- **[Node Types](user-guide/node-types.md)**: Learn about all available nodes
- **[Custom Nodes](user-guide/custom-nodes.md)**: Create your own node types
- **[Debugging](user-guide/debugging.md)**: Advanced debugging techniques
- **[Deployment](deployment/README.md)**: Production deployment strategies

## Troubleshooting

### Common Issues

**Editor won't start**
```bash
# Check if all dependencies are installed
make check-deps

# Rebuild the project
make clean && make build
```

**Compilation errors**
- Check node connections are valid
- Verify all required inputs are connected
- Review error messages in the validation panel

**Deployment fails**
- Ensure BaaLS node is running
- Check network connectivity
- Verify contract validation passes

### Getting Help

- **Documentation**: Check the relevant guides
- **Issues**: [GitHub Issues](https://github.com/kdsmith18542/CanvasContracts/issues)
- **Community**: [Discord Server](https://discord.gg/canvascontracts)

## What's Next?

Congratulations! You've successfully created and deployed your first visual smart contract. The platform offers many advanced features:

- **Advanced Patterns**: Complex contract architectures
- **Custom Nodes**: Extend the platform with your own logic
- **Production Deployment**: Blue-green deployments and canary releases
- **Monitoring**: Comprehensive metrics and alerting
- **SDK**: Integrate with your own applications

Ready to explore more? Check out the [User Guide](user-guide/visual-editor.md) for detailed information about all features. 