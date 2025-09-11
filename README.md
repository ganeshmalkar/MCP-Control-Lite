# 🚀 MCP Control

**The Ultimate Model Context Protocol Server Management Tool**

[![Version](https://img.shields.io/badge/version-1.1.0-blue.svg)](https://github.com/Chykalophia/MCP-Control-Lite/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](#installation)

MCP Control is a powerful desktop application that simplifies the management of Model Context Protocol (MCP) servers. Discover, install, configure, and monitor MCP servers across multiple AI applications with an intuitive graphical interface.

![MCP Control Screenshot](https://via.placeholder.com/800x500/1a1a1a/ffffff?text=MCP+Control+Interface)

## ✨ Features

### 🔍 **Smart Package Discovery**
- **Real-time NPM Search**: Direct integration with NPM registry using `npm search --json`
- **GitHub Repository Search**: Browse community MCP servers with stars, descriptions, and topics
- **PulseMCP Integration**: Access curated MCP servers from the PulseMCP registry
- **Intelligent Filtering**: Smart keyword extraction and source-specific tagging

### 📦 **Seamless Installation**
- **One-Click Install**: Install MCP servers directly from the interface
- **Real-time Status**: Live installation progress with visual feedback
- **Automatic Detection**: Installed packages automatically appear in your server list
- **Cross-Platform**: Works with Claude Desktop, Cursor, VS Code, Zed, and more

### 🎛️ **Server Management**
- **Unified Dashboard**: Manage all your MCP servers from one place
- **Configuration Sync**: Bi-directional synchronization with application configs
- **Status Monitoring**: Real-time server health and performance tracking
- **Easy Updates**: Keep your MCP servers up-to-date effortlessly

### 📊 **Comprehensive Logging**
- **Installation Audit Trail**: Track all package installations with detailed logs
- **Error Diagnostics**: Comprehensive error reporting and debugging information
- **Activity Monitoring**: Monitor server activity and performance metrics
- **Configurable Logging**: Enable/disable logging based on your needs

### 🎨 **Modern Interface**
- **Intuitive Design**: Clean, modern interface built with React and Tauri
- **Dark/Light Themes**: Customizable appearance to match your workflow
- **Responsive Layout**: Optimized for different screen sizes and resolutions
- **Keyboard Shortcuts**: Efficient navigation with keyboard shortcuts

## 🚀 Quick Start

### Installation

#### macOS
1. Download the latest `MCP Control_x.x.x_aarch64.dmg` from [Releases](https://github.com/Chykalophia/MCP-Control-Lite/releases)
2. Open the DMG file and drag MCP Control to Applications
3. Launch MCP Control from Applications or Spotlight

#### Windows & Linux
Coming soon! Follow the repository for updates.

### First Launch

1. **Discover Servers**: Browse the Discover tab to find MCP servers
2. **Search & Filter**: Use the search bar to find specific functionality
3. **Install Packages**: Click "Install" on any package you want to add
4. **Configure Applications**: Go to Settings to configure your AI applications
5. **Monitor Activity**: Check the Logs tab for installation and activity logs

## 🛠️ Development

### Prerequisites
- Node.js 18+ and npm
- Rust 1.77.2+
- Tauri CLI

### Setup
```bash
# Clone the repository
git clone https://github.com/Chykalophia/MCP-Control-Lite.git
cd MCP-Control-Lite

# Install dependencies
npm install

# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
npm run tauri dev
```

### Building
```bash
# Build for production
npm run build
cd src-tauri
cargo tauri build
```

## 📖 Usage Examples

### Discovering Weather MCP Servers
1. Open the **Discover** tab
2. Search for "weather" in the search bar
3. Browse results from NPM, GitHub, and PulseMCP
4. Click on repository links to learn more
5. Install with one click

### Managing Server Configurations
1. Go to the **Servers** tab to see all installed servers
2. Configure server settings and parameters
3. Enable/disable servers for specific applications
4. Monitor server status and performance

### Viewing Installation Logs
1. Navigate to the **Logs** tab
2. Filter by log level (Error, Warning, Info, Debug)
3. Search through logs for specific events
4. Export logs for troubleshooting

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏢 About Chykalophia

MCP Control is developed by [**Chykalophia**](https://chykalophia.com), a leading technology consultancy specializing in AI integration and custom software solutions.

### 🚀 **Need Custom AI Solutions?**

Our team has helped businesses achieve **measurable 3x growth** through strategic AI implementation and custom development solutions. From MCP server development to complete AI workflow automation, we deliver results that transform your business.

**Ready to 3x your business growth?** [**Contact us today**](https://chykalophia.com) to discuss your custom AI solution needs.

## 👨‍💻 Core Contributor

**Peter Krzyzek** - Lead Developer & Fractional CTO  
🌐 Website: [piotrkrzyzek.com](https://piotrkrzyzek.com)  
💼 LinkedIn: [linkedin.com/in/gopeter](https://linkedin.com/in/gopeter)  

**Need a Fractional CTO?** Peter specializes in scaling engineering teams and architecting robust systems. Available for fractional CTO engagements through his website.

---

## 🔗 Links

- **Website**: [chykalophia.com](https://chykalophia.com)
- **Documentation**: [Coming Soon]
- **Issues**: [GitHub Issues](https://github.com/Chykalophia/MCP-Control-Lite/issues)
- **Releases**: [GitHub Releases](https://github.com/Chykalophia/MCP-Control-Lite/releases)

## ⭐ Support

If you find MCP Control useful, please consider:
- ⭐ Starring this repository
- 🐛 Reporting bugs and issues
- 💡 Suggesting new features
- 🤝 Contributing to the codebase

---

<div align="center">
  <strong>Built with ❤️ by <a href="https://chykalophia.com">Chykalophia</a></strong>
</div>
