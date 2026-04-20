# PuerceNote

<h1 align="center" style="border-bottom: none">
    <b>PuerceNote</b><br>
    ⭐️  AI-Powered Privacy-First Note Taking & Collaboration  ⭐️
</h1>

<p align="center">
The ultimate workspace for professionals, teams, and enterprises who demand privacy, intelligence, and control.
</p>

<p align="center">
<a href="https://opensource.org/licenses/AGPL-3.0"><img src="https://img.shields.io/badge/license-AGPL%203.0-green.svg" alt="License: AGPL 3.0"></a>
<a href="https://github.com/JoeLeung2018/puercenote"><img src="https://img.shields.io/github/stars/JoeLeung2018/puercenote.svg?style=flat&logo=github&colorB=deeppink&label=stars"></a>
<a href="https://github.com/JoeLeung2018/puercenote"><img src="https://img.shields.io/badge/status-v1.0--beta-blue.svg"></a>
</p>

## What is PuerceNote?

PuerceNote is a modern, AI-powered workspace platform designed for individuals and teams who value:

- **🔐 Privacy First**: Your data stays on your device. Zero cloud dependency. Local-first architecture.
- **🤖 Intelligent AI**: Integrated Llama-2 9B for summarization, tagging, semantic search – all running locally or on your infrastructure
- **💰 Transparent Pricing**: Token-based billing with full cost transparency. No hidden fees.
- **🌐 Web3 Ready**: Native USDC/USDT payment support on Polygon for global accessibility
- **🔗 Multi-Payment**: Credit cards (Lemon Squeezy) + Blockchain (Polygon) for maximum flexibility

## Key Features

### 🎯 Core Capabilities
- **Documents & Databases**: Full-featured note-taking with relational databases
- **Real-time Collaboration**: Synchronous editing for teams
- **AI-Powered Insights**: Summarization, tagging, semantic search powered by local AI
- **Privacy-Preserving**: TinyLlama 1.1B for sensitive content detection (stays local)
- **Mobile Ready**: Native iOS/Android apps

### 💡 AI Features (Subscription Required)
- **AI Summarization**: Auto-summarize documents and notes
- **Smart Tagging**: Intelligent categorization powered by embeddings
- **Semantic Search**: Find notes by meaning, not just keywords
- **AI Workspace Insights**: Analytics on your knowledge base

### 🛡️ Enterprise Features
- **Self-Hosted AI**: Run Llama-2 9B on your own GPU infrastructure
- **Token Billing**: Pay only for what you use, with predictable costs
- **HIPAA Compliant**: Option for private deployment
- **Custom Branding**: White-label deployment available

## Pricing Plans

### Free
- Documents & Databases
- Local markdown editor
- Real-time collaboration
- Community support
- **Price**: $0/month

### Professional (¥69/month or $9.99)
- Everything in Free
- AI Summarization (100,000 tokens/month)
- Smart Tagging
- Semantic Search
- Email support
- **Price**: ¥69/month (≈ $9.99)

### Team (¥199/month or $29.99)
- Everything in Professional
- 5× token quota (500,000 tokens/month)
- Team collaboration features
- API access
- Priority support
- **Price**: ¥199/month (≈ $29.99)

**Payment Methods**:
- 💳 Credit Card (Visa, Mastercard, Amex) via Lemon Squeezy
- 🔗 Blockchain (USDC/USDT on Polygon) for global users

## Technology Stack

### Frontend
- **Framework**: Flutter (Dart) for cross-platform UI
- **Communication**: Dart FFI to Rust backend (low-latency local architecture)
- **Platforms**: iOS, Android, macOS, Windows, Linux, Web

### Backend
- **Runtime**: Rust with 32 carefully selected crates
- **Database**: SQLite client-side with Diesel ORM
- **Async**: Tokio for high-performance networking

### AI
- **Model**: Llama-2 9B (self-hosted via vLLM)
- **Local Privacy Filter**: TinyLlama 1.1B Q4_K quantization
- **Embeddings**: DistilBERT (384-dimensional) for semantic search
- **Vector DB**: LanceDB for efficient similarity search
- **Token Counting**: Local heuristic algorithm with TByte compensation

### Infrastructure
- **Deployment**: Self-hosted on RTX 4060 GPUs in China
- **Latency**: <50ms for primary user base
- **Network**: frpc tunneling for secure GPU access
- **Cost**: $520/month operational cost (optimized)

### Blockchain (Optional)
- **Network**: Polygon (Mumbai testnet → mainnet)
- **Tokens**: USDC/USDT
- **Integration**: WalletConnect 2.0
- **Smart Contracts**: Custom subscription contract

## Open Source & Community

PuerceNote is **100% open source** under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

### Why AGPL-3.0?
- Ensures all derivatives remain open source
- Protects the community while enabling commercial use
- Fair balance between business and freedom

### Contributing
We welcome contributions from the community! See [CONTRIBUTING.md](./doc/CONTRIBUTING.md) for guidelines.

### Community & Support
- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions and share ideas
- **Email**: support@puerce.com

## Getting Started

### Self-Hosted Deployment
```bash
# Clone the repository
git clone https://github.com/JoeLeung2018/puercenote.git
cd puercenote

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Build and run
./install.sh
```

See [DEVELOPMENT.md](./DEVELOPMENT.md) for detailed setup instructions.

### Cloud Deployment (Coming Soon)
Managed hosting will be available at v1.1.

## Development Status

**Version**: 1.0-beta  
**Target Launch**: Q1 2025  
**Status**: Active development

**Phase 1 (Complete)**: Foundation & Architecture  
**Phase 2 (In Progress)**: Payment Integration & AI Billing  
**Phase 3 (Planned)**: Web3 Integration & Enterprise Features  

See [ROADMAP.md](./ROADMAP.md) for detailed plans.

## Safety & Security

- ✅ **AGPL-3.0 Compliant**: Fully open source
- ✅ **Local First**: No mandatory cloud dependencies
- ✅ **Encrypted**: TLS 1.3 for all network communication
- ✅ **Auditable**: Complete source code transparency
- ✅ **Regular Updates**: Security patches within 24-48 hours

## License

This project is licensed under the **GNU Affero General Public License v3.0**. See [LICENSE](./LICENSE) for details.

**Commercial Services License**: For proprietary deployments or closed-source modifications, contact us at: enterprise@puerce.com

## Contact & Support

- 🌐 **Website**: (coming soon)
- 📧 **Email**: support@puerce.com
- 💬 **Discord**: (coming soon)
- 📱 **Twitter**: @puercenote

---

<p align="center">
  <b>Made with ❤️ for privacy-conscious professionals and teams</b>
</p>

<p align="center">
  <b>PuerceNote © 2026. All rights reserved.</b>
</p>
