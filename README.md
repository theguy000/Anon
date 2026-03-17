<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Anon Browser" width="128" />
</p>

<h1 align="center">Anon Browser</h1>

<p align="center">
  A lightweight, open-source instance manager for Camoufox, a privacy-focused browser built on Firefox.
</p>

## Features

- **Instance Management**: Create, manage, and delete multiple isolated browser instances
- **Proxy Support**: Configure proxy settings per instance
- **Quick Launch**: One-click launch of browser instances
- **Modern UI**: Clean, minimalist interface with dark mode
- **Cross-Platform**: Built with Tauri for Windows, macOS, and Linux

## Installation

### Prerequisites

- Rust (1.70+)
- Node.js (18+)
- pnpm (8+)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/anon-instance-manager.git
   cd anon-instance-manager
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Run the application:
   ```bash
   pnpm tauri dev
   ```

## License

MIT
