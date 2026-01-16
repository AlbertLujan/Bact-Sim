# 🦠 Bacterial Ecosystem Simulator

A real-time evolutionary ecosystem simulation written in **Rust** using [macroquad](https://github.com/not-fl3/macroquad).

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)

## ✨ Features

- **Real-time Evolution** - Bacteria mutate and evolve traits like speed, size, and sensing radius
- **Predator-Prey Dynamics** - Predators hunt bacteria, bacteria flee and seek food
- **Energy System** - Metabolism costs energy, eating replenishes it
- **Interactive UI** - Adjust simulation parameters with sliders in real-time
- **Live Statistics** - Population graphs updated every frame
- **High Performance** - Smooth 60+ FPS with hundreds of agents

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/AlbertLujan/bact-sim.git
cd bact-sim

# Run the simulation
cargo run --release
```

## 📋 Requirements

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs)
- Works on Windows, Linux, and macOS

## 🎮 Controls

| Key | Action |
|-----|--------|
| `TAB` | Show/Hide UI panel |
| `SPACE` | Pause/Resume simulation |

## 🧬 Genetic System

Each bacterium carries DNA that determines:

| Trait | Description |
|-------|-------------|
| **Speed** | Movement velocity (higher = faster but more energy cost) |
| **Size** | Body radius (larger = easier to catch food but slower) |
| **Sense Radius** | Detection range for food and predators |
| **Color** | Inherited with slight mutations |

When bacteria reproduce, offspring inherit mutated versions of parent traits.

## 📊 UI Parameters

The simulation panel lets you control:

- **Food/Frame** - Food spawn rate
- **Sim. Speed** - Simulation speed multiplier
- **Mutation Rate** - Chance of trait mutations
- **Mutation Strength** - Magnitude of mutations
- **Initial Energy** - Starting energy for new bacteria
- **Reproduction** - Energy threshold for reproduction
- **Pred. Repro.** - Predator reproduction threshold

## 🏗️ Project Structure

```
bact-sim/
├── src/
│   └── main.rs      # All simulation code
├── Cargo.toml       # Dependencies
└── README.md
```

## 📦 Dependencies

- [macroquad](https://crates.io/crates/macroquad) - Cross-platform game framework
- [rand](https://crates.io/crates/rand) - Random number generation

## 🤝 Contributing

Contributions are welcome! Feel free to:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [macroquad](https://github.com/not-fl3/macroquad) by [@not-fl3](https://github.com/not-fl3)
- Inspired by evolutionary simulation concepts

---

**Made with ❤️ in Rust**
