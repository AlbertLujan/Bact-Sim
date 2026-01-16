# Bact-Sim

A bacteria ecosystem simulator with predators, evolution, and real-time stats. Built in Rust with macroquad.

## Run it

```bash
cargo run --release
```

## What's going on

Bacteria spawn, look for food, and try not to get eaten by predators. When they eat enough, they reproduce and pass on their genes (speed, size, sensing range) with small mutations. Over time, you can watch natural selection happen - faster bacteria tend to survive better, but they also burn more energy.

Predators hunt bacteria. If they eat enough, they reproduce too. If bacteria go extinct, the simulation auto-respawns some to keep things interesting.

## Controls

- **TAB** - toggle the UI panel
- **SPACE** - pause/resume

## The UI panel

You can tweak everything while it runs:
- Food spawn rate
- Simulation speed  
- Mutation rate and strength
- Energy thresholds for reproduction

The graphs at the bottom show population over time, average speed, average size, and predator count.

## Dependencies

Just macroquad and rand. Check `Cargo.toml`.

## License

MIT
