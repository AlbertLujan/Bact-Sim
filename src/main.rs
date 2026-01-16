use macroquad::prelude::*;
use ::rand::Rng;

const SCREEN_WIDTH: f32 = 1400.0;
const SCREEN_HEIGHT: f32 = 900.0;
const GRAPH_HEIGHT: f32 = 200.0;
const UI_PANEL_WIDTH: f32 = 320.0;
const MAX_HISTORY: usize = 300;

// Mutable simulation parameters
struct SimulationParams {
    food_growth_rate: f32,
    max_food: usize,
    mutation_rate: f32,
    mutation_strength: f32,
    reproduction_threshold: f32,
    initial_energy: f32,
    speed_multiplier: f32,
    predator_count: f32,
    predator_reproduction_threshold: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            food_growth_rate: 2.0,
            max_food: 1000,
            mutation_rate: 0.1,
            mutation_strength: 0.1,
            reproduction_threshold: 150.0,
            initial_energy: 100.0,
            speed_multiplier: 1.0,
            predator_count: 5.0,
            predator_reproduction_threshold: 200.0,
        }
    }
}

// Historical stats for graphing
struct Stats {
    population_history: Vec<f32>,
    avg_speed_history: Vec<f32>,
    avg_size_history: Vec<f32>,
    predator_history: Vec<f32>,
}

impl Stats {
    fn new() -> Self {
        Self {
            population_history: Vec::new(),
            avg_speed_history: Vec::new(),
            avg_size_history: Vec::new(),
            predator_history: Vec::new(),
        }
    }

    fn push(&mut self, pop: f32, speed: f32, size: f32, predators: f32) {
        self.population_history.push(pop);
        self.avg_speed_history.push(speed);
        self.avg_size_history.push(size);
        self.predator_history.push(predators);

        // Keep only last MAX_HISTORY entries
        if self.population_history.len() > MAX_HISTORY {
            self.population_history.remove(0);
            self.avg_speed_history.remove(0);
            self.avg_size_history.remove(0);
            self.predator_history.remove(0);
        }
    }
}

struct UIState {
    show_ui: bool,
    paused: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            show_ui: true,
            paused: false,
        }
    }
}

#[derive(Clone)]
struct DNA {
    speed: f32,
    size: f32,
    sense_radius: f32,
    color: Color,
}

impl DNA {
    fn random() -> Self {
        let mut rng = ::rand::thread_rng();
        Self {
            speed: rng.gen_range(1.0f32..3.0f32),
            size: rng.gen_range(3.0f32..8.0f32),
            sense_radius: rng.gen_range(20.0f32..60.0f32),
            color: Color::new(
                rng.gen_range(0.2f32..1.0f32),
                rng.gen_range(0.2f32..1.0f32),
                rng.gen_range(0.2f32..1.0f32),
                0.9f32,
            ),
        }
    }

    fn mutate(&self, params: &SimulationParams) -> Self {
        let mut rng = ::rand::thread_rng();
        
        let new_speed = if rng.gen_bool(params.mutation_rate as f64) {
            let change = rng.gen_range(-params.mutation_strength..params.mutation_strength);
            (self.speed * (1.0 + change)).clamp(0.1, 100.0)
        } else {
            self.speed
        };
        
        let new_size = if rng.gen_bool(params.mutation_rate as f64) {
            let change = rng.gen_range(-params.mutation_strength..params.mutation_strength);
            (self.size * (1.0 + change)).clamp(0.1, 100.0)
        } else {
            self.size
        };
        
        let new_sense = if rng.gen_bool(params.mutation_rate as f64) {
            let change = rng.gen_range(-params.mutation_strength..params.mutation_strength);
            (self.sense_radius * (1.0 + change)).clamp(0.1, 100.0)
        } else {
            self.sense_radius
        };

        let new_r = (self.color.r + rng.gen_range(-0.05f32..0.05f32)).clamp(0.2, 1.0);
        let new_g = (self.color.g + rng.gen_range(-0.05f32..0.05f32)).clamp(0.2, 1.0);
        let new_b = (self.color.b + rng.gen_range(-0.05f32..0.05f32)).clamp(0.2, 1.0);

        Self {
            speed: new_speed,
            size: new_size,
            sense_radius: new_sense,
            color: Color::new(new_r, new_g, new_b, 0.9f32),
        }
    }
}

struct Bacterium {
    pos: Vec2,
    vel: Vec2,
    dna: DNA,
    energy: f32,
    age: f32,
}

impl Bacterium {
    fn new(pos: Vec2, initial_energy: f32) -> Self {
        let mut rng = ::rand::thread_rng();
        let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
        let dna = DNA::random();
        Self {
            pos,
            vel: vec2(angle.cos(), angle.sin()),
            dna,
            energy: initial_energy,
            age: 0.0,
        }
    }

    fn update(&mut self, screen_w: f32, screen_h: f32, food: &Vec<Vec2>, speed_mult: f32, predators: &Vec<Predator>) {
        let mut rng = ::rand::thread_rng();
        
        // Movement physics
        self.pos += self.vel * self.dna.speed * speed_mult;
        
        // Bounce off walls
        if self.pos.x < 0.0 || self.pos.x > screen_w {
            self.vel.x *= -1.0;
            self.pos.x = self.pos.x.clamp(0.0, screen_w);
        }
        if self.pos.y < 0.0 || self.pos.y > screen_h {
            self.vel.y *= -1.0;
            self.pos.y = self.pos.y.clamp(0.0, screen_h);
        }

        // Check for nearby predators and flee
        let mut flee_dir = vec2(0.0, 0.0);
        for p in predators {
            let dist = self.pos.distance(p.pos);
            if dist < 80.0 && dist > 0.1 {
                let away = (self.pos - p.pos).normalize();
                flee_dir += away / dist;
            }
        }
        
        if flee_dir.length() > 0.1 {
            // Flee from predators
            self.vel = (self.vel + flee_dir.normalize() * 0.5).normalize();
        } else {
            // Normal behavior: random jitter / steering
            let jitter_angle = rng.gen_range(-0.2f32..0.2f32);
            let new_angle = self.vel.y.atan2(self.vel.x) + jitter_angle;
            self.vel = vec2(new_angle.cos(), new_angle.sin());

            // Find nearest food
            if !food.is_empty() {
                 let mut nearest_dist = f32::MAX;
                 let mut nearest_idx = Option::None;

                 for (i, f) in food.iter().enumerate() {
                     let d = self.pos.distance(*f);
                     if d < self.dna.sense_radius && d < nearest_dist {
                         nearest_dist = d;
                         nearest_idx = Some(i);
                     }
                 }

                 if let Some(idx) = nearest_idx {
                     // Steer towards food
                     let target = food[idx];
                     let dir = (target - self.pos).normalize();
                     self.vel = (self.vel + dir * 0.2).normalize();
                 }
            }
        }

        // Metabolism
        let cost = (self.dna.speed * self.dna.speed * self.dna.size * 0.005) + 0.1;
        self.energy -= cost * speed_mult;
        self.age += 1.0;
    }
}

struct Predator {
    pos: Vec2,
    vel: Vec2,
    energy: f32,
    speed: f32,
    size: f32,
    sense_radius: f32,
}

impl Predator {
    fn new(pos: Vec2) -> Self {
        let mut rng = ::rand::thread_rng();
        let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
        Self {
            pos,
            vel: vec2(angle.cos(), angle.sin()),
            energy: 150.0,
            speed: 2.5,
            size: 12.0,
            sense_radius: 100.0,
        }
    }

    fn update(&mut self, screen_w: f32, screen_h: f32, bacteria: &Vec<Bacterium>, speed_mult: f32) {
        let mut rng = ::rand::thread_rng();
        
        // Movement physics
        self.pos += self.vel * self.speed * speed_mult;
        
        // Bounce off walls
        if self.pos.x < 0.0 || self.pos.x > screen_w {
            self.vel.x *= -1.0;
            self.pos.x = self.pos.x.clamp(0.0, screen_w);
        }
        if self.pos.y < 0.0 || self.pos.y > screen_h {
            self.vel.y *= -1.0;
            self.pos.y = self.pos.y.clamp(0.0, screen_h);
        }

        // Hunt nearest bacterium
        if !bacteria.is_empty() {
            let mut nearest_dist = f32::MAX;
            let mut nearest_pos = None;

            for b in bacteria {
                let d = self.pos.distance(b.pos);
                if d < self.sense_radius && d < nearest_dist {
                    nearest_dist = d;
                    nearest_pos = Some(b.pos);
                }
            }

            if let Some(target) = nearest_pos {
                // Chase bacterium
                let dir = (target - self.pos).normalize();
                self.vel = (self.vel + dir * 0.3).normalize();
            } else {
                // Random wandering
                let jitter_angle = rng.gen_range(-0.15f32..0.15f32);
                let new_angle = self.vel.y.atan2(self.vel.x) + jitter_angle;
                self.vel = vec2(new_angle.cos(), new_angle.sin());
            }
        }

        // Metabolism - predators consume more energy
        let cost = 0.2;
        self.energy -= cost * speed_mult;
    }
}

// UI Helper Functions
fn draw_section_header(x: f32, y: f32, width: f32, text: &str, color: Color) -> f32 {
    draw_text(text, x, y, 20.0, color);
    draw_line(x, y + 5.0, x + width, y + 5.0, 2.0, color);
    y + 25.0
}

fn draw_slider(x: f32, y: f32, width: f32, label: &str, value: &mut f32, min: f32, max: f32, unit: &str) {
    let height = 22.0;
    let (mouse_x, mouse_y) = mouse_position();
    let mouse_down = is_mouse_button_down(MouseButton::Left);
    
    // Draw label with unit
    let label_text = format!("{}", label);
    draw_text(&label_text, x, y - 5.0, 16.0, WHITE);
    
    // Draw track
    draw_rectangle(x, y, width, height, Color::new(0.15, 0.15, 0.15, 0.9));
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.4, 0.4, 0.4, 0.8));
    
    // Calculate slider position
    let norm = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let handle_x = x + norm * width;
    
    // Draw fill with gradient effect
    draw_rectangle(x, y, norm * width, height, Color::new(0.2, 0.5, 0.9, 0.9));
    
    // Draw handle
    draw_rectangle(handle_x - 6.0, y - 2.0, 12.0, height + 4.0, Color::new(0.9, 0.95, 1.0, 1.0));
    draw_rectangle_lines(handle_x - 6.0, y - 2.0, 12.0, height + 4.0, 1.0, Color::new(0.3, 0.3, 0.3, 1.0));
    
    // Draw value text with unit
    let value_text = if unit == "%" {
        format!("{:.0}{}", *value * 100.0, unit)
    } else if unit == "x" {
        format!("{:.1}{}", value, unit)
    } else {
        format!("{:.0} {}", value, unit)
    };
    draw_text(&value_text, x + width + 10.0, y + 16.0, 16.0, Color::new(0.8, 1.0, 0.8, 1.0));
    
    // Check for interaction
    if mouse_down && mouse_x >= x && mouse_x <= x + width && mouse_y >= y - 5.0 && mouse_y <= y + height + 5.0 {
        let new_norm = ((mouse_x - x) / width).clamp(0.0, 1.0);
        *value = min + new_norm * (max - min);
    }
}

fn draw_button(x: f32, y: f32, width: f32, height: f32, label: &str, color: Color) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
    
    let hovered = mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height;
    
    let btn_color = if hovered {
        Color::new((color.r * 1.3).min(1.0), (color.g * 1.3).min(1.0), (color.b * 1.3).min(1.0), color.a)
    } else {
        color
    };
    
    draw_rectangle(x, y, width, height, btn_color);
    draw_rectangle_lines(x, y, width, height, 2.0, WHITE);
    
    // Center text
    let text_size = 20.0;
    let text_dims = measure_text(label, None, text_size as u16, 1.0);
    draw_text(label, x + (width - text_dims.width) / 2.0, y + (height + text_size) / 2.0 - 2.0, text_size, WHITE);
    
    hovered && mouse_clicked
}

fn draw_line_graph(x: f32, y: f32, width: f32, height: f32, data: &Vec<f32>, color: Color, label: &str, max_val: Option<f32>) {
    // Background with slight gradient
    draw_rectangle(x, y, width, height, Color::new(0.08, 0.08, 0.1, 0.85));
    draw_rectangle_lines(x, y, width, height, 2.0, Color::new(0.3, 0.3, 0.3, 0.6));
    
    if data.is_empty() {
        return;
    }
    
    // Find max value for scaling
    let max_value = max_val.unwrap_or_else(|| {
        data.iter().cloned().fold(f32::NEG_INFINITY, f32::max).max(1.0)
    });
    
    // Draw grid lines
    for i in 0..=4 {
        let grid_y = y + (i as f32 / 4.0) * height;
        draw_line(x, grid_y, x + width, grid_y, 1.0, Color::new(0.2, 0.2, 0.2, 0.4));
    }
    
    // Draw data points with glow effect
    let step = width / (MAX_HISTORY as f32);
    for i in 0..data.len().saturating_sub(1) {
        let x1 = x + (i as f32) * step;
        let y1 = y + height - (data[i] / max_value * height).min(height);
        let x2 = x + ((i + 1) as f32) * step;
        let y2 = y + height - (data[i + 1] / max_value * height).min(height);
        
        // Glow effect
        draw_line(x1, y1, x2, y2, 4.0, Color::new(color.r, color.g, color.b, 0.3));
        draw_line(x1, y1, x2, y2, 2.0, color);
    }
    
    // Draw label and current value with background
    draw_rectangle(x + 3.0, y + 3.0, 150.0, 42.0, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_text(label, x + 8.0, y + 20.0, 18.0, WHITE);
    if let Some(last) = data.last() {
        let value_text = format!("{:.0}", last);
        draw_text(&value_text, x + 8.0, y + 38.0, 22.0, color);
    }
    
    // Draw max value
    let max_text = format!("max: {:.0}", max_value);
    draw_text(&max_text, x + width - 70.0, y + 18.0, 15.0, Color::new(0.7, 0.7, 0.7, 1.0));
}

fn draw_ui_panel(params: &mut SimulationParams, ui_state: &mut UIState, bacteria_count: usize, food_count: usize, predator_count: usize) {
    let panel_x = SCREEN_WIDTH - UI_PANEL_WIDTH;
    let panel_y = 0.0;
    
    // Dark background with border
    draw_rectangle(panel_x, panel_y, UI_PANEL_WIDTH, SCREEN_HEIGHT, Color::new(0.02, 0.02, 0.03, 0.95));
    draw_line(panel_x, 0.0, panel_x, SCREEN_HEIGHT, 3.0, Color::new(0.3, 0.4, 0.5, 0.8));
    
    let mut current_y = 25.0;
    let slider_width = UI_PANEL_WIDTH - 140.0;
    let x_offset = panel_x + 20.0;
    
    // Main Title
    draw_text("SIMULATION", x_offset, current_y, 28.0, Color::new(0.4, 0.7, 1.0, 1.0));
    current_y += 40.0;
    
    // STATS SECTION
    current_y = draw_section_header(x_offset, current_y, slider_width + 60.0, "📊 POPULATIONS", Color::new(0.3, 0.9, 0.3, 1.0));
    draw_text(&format!("🦠 Bacteria: {}", bacteria_count), x_offset, current_y, 18.0, Color::new(0.5, 1.0, 0.5, 1.0));
    current_y += 23.0;
    draw_text(&format!("🍃 Food: {}", food_count), x_offset, current_y, 18.0, Color::new(0.3, 0.9, 0.5, 1.0));
    current_y += 23.0;
    draw_text(&format!("🦖 Predators: {}", predator_count), x_offset, current_y, 18.0, Color::new(1.0, 0.4, 0.3, 1.0));
    current_y += 45.0;
    
    // SIMULATION SECTION
    current_y = draw_section_header(x_offset, current_y, slider_width + 60.0, "⚙️ SIMULATION", Color::new(0.5, 0.8, 1.0, 1.0));
    draw_slider(x_offset, current_y, slider_width, "Food/Frame", &mut params.food_growth_rate, 0.0, 10.0, "/f");
    current_y += 50.0;
    
    draw_slider(x_offset, current_y, slider_width, "Sim. Speed", &mut params.speed_multiplier, 0.1, 3.0, "x");
    current_y += 55.0;
    
    // EVOLUTION SECTION
    current_y = draw_section_header(x_offset, current_y, slider_width + 60.0, "🧬 EVOLUTION", Color::new(0.9, 0.5, 0.9, 1.0));
    draw_slider(x_offset, current_y, slider_width, "Mutation Rate", &mut params.mutation_rate, 0.0, 0.5, "%");
    current_y += 50.0;
    
    draw_slider(x_offset, current_y, slider_width, "Mutation Str.", &mut params.mutation_strength, 0.0, 0.5, "%");
    current_y += 55.0;
    
    // ENERGY SECTION
    current_y = draw_section_header(x_offset, current_y, slider_width + 60.0, "⚡ ENERGY", Color::new(1.0, 0.9, 0.3, 1.0));
    draw_slider(x_offset, current_y, slider_width, "Initial Energy", &mut params.initial_energy, 50.0, 200.0, "");
    current_y += 50.0;
    
    draw_slider(x_offset, current_y, slider_width, "Reproduction", &mut params.reproduction_threshold, 50.0, 300.0, "");
    current_y += 55.0;
    
    // PREDATORS SECTION
    current_y = draw_section_header(x_offset, current_y, slider_width + 60.0, "🦖 PREDATORS", Color::new(1.0, 0.4, 0.3, 1.0));
    draw_slider(x_offset, current_y, slider_width, "Pred. Repro.", &mut params.predator_reproduction_threshold, 100.0, 400.0, "");
    current_y += 60.0;
    
    // CONTROLS
    let btn_width = (UI_PANEL_WIDTH - 50.0) / 2.0;
    if draw_button(x_offset, current_y, btn_width, 40.0, 
                    if ui_state.paused { "▶ PLAY" } else { "⏸ PAUSE" },
                    Color::new(0.2, 0.5, 0.9, 0.95)) {
        ui_state.paused = !ui_state.paused;
    }
    
    // Instructions at bottom
    current_y = SCREEN_HEIGHT - 70.0;
    draw_rectangle(x_offset - 10.0, current_y - 15.0, UI_PANEL_WIDTH - 20.0, 60.0, Color::new(0.1, 0.1, 0.1, 0.5));
    draw_text("CONTROLS:", x_offset, current_y, 16.0, Color::new(0.7, 0.7, 0.7, 1.0));
    current_y += 20.0;
    draw_text("TAB   →  Show/Hide UI", x_offset, current_y, 14.0, DARKGRAY);
    current_y += 18.0;
    draw_text("SPACE →  Pause", x_offset, current_y, 14.0, DARKGRAY);
}

fn draw_graphs_panel(stats: &Stats) {
    let panel_y = SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0;
    let graph_width = (SCREEN_WIDTH - UI_PANEL_WIDTH - 50.0) / 4.0;
    let x_start = 10.0;
    
    // Population graph
    draw_line_graph(x_start, panel_y, graph_width, GRAPH_HEIGHT, 
                     &stats.population_history, 
                     Color::new(0.3, 1.0, 0.3, 1.0), 
                     "🦠 Bacteria", None);
    
    // Predator graph
    draw_line_graph(x_start + graph_width + 10.0, panel_y, graph_width, GRAPH_HEIGHT,
                     &stats.predator_history,
                     Color::new(1.0, 0.4, 0.3, 1.0),
                     "🦖 Predators", None);
    
    // Speed graph
    draw_line_graph(x_start + (graph_width + 10.0) * 2.0, panel_y, graph_width, GRAPH_HEIGHT,
                     &stats.avg_speed_history,
                     Color::new(0.9, 0.7, 0.2, 1.0),
                     "⚡ Speed", Some(5.0));
    
    // Size graph
    draw_line_graph(x_start + (graph_width + 10.0) * 3.0, panel_y, graph_width, GRAPH_HEIGHT,
                     &stats.avg_size_history,
                     Color::new(0.5, 0.5, 1.0, 1.0),
                     "📏 Size", Some(15.0));
}

#[macroquad::main("Bacterial Ecosystem")]
async fn main() {
    request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);

    let mut params = SimulationParams::default();
    let mut ui_state = UIState::default();
    let mut stats = Stats::new();
    
    let mut bacteria: Vec<Bacterium> = Vec::new();
    let mut predators: Vec<Predator> = Vec::new();
    let mut food: Vec<Vec2> = Vec::new();

    // Initialize population
    let initial_bacteria = 50;
    for _ in 0..initial_bacteria {
        bacteria.push(Bacterium::new(vec2(
            macroquad::rand::gen_range(0.0f32, SCREEN_WIDTH - UI_PANEL_WIDTH),
            macroquad::rand::gen_range(0.0f32, SCREEN_HEIGHT - GRAPH_HEIGHT),
        ), params.initial_energy));
    }

    // Initialize predators
    let initial_predators = params.predator_count as usize;
    for _ in 0..initial_predators {
        predators.push(Predator::new(vec2(
            macroquad::rand::gen_range(0.0f32, SCREEN_WIDTH - UI_PANEL_WIDTH),
            macroquad::rand::gen_range(0.0f32, SCREEN_HEIGHT - GRAPH_HEIGHT),
        )));
    }

    // Initialize food
    let initial_food = 200;
    for _ in 0..initial_food {
        food.push(vec2(
            macroquad::rand::gen_range(0.0f32, SCREEN_WIDTH - UI_PANEL_WIDTH),
            macroquad::rand::gen_range(0.0f32, SCREEN_HEIGHT - GRAPH_HEIGHT),
        ));
    }

    loop {
        let sim_w = SCREEN_WIDTH - UI_PANEL_WIDTH;
        let sim_h = SCREEN_HEIGHT - GRAPH_HEIGHT;

        // Handle input
        if is_key_pressed(KeyCode::Tab) {
            ui_state.show_ui = !ui_state.show_ui;
        }
        if is_key_pressed(KeyCode::Space) {
            ui_state.paused = !ui_state.paused;
        }

        // Update Game State (only if not paused)
        if !ui_state.paused {
            // Add random food
            if food.len() < params.max_food {
                let to_add = params.food_growth_rate as usize;
                for _ in 0..to_add {
                     food.push(vec2(
                        macroquad::rand::gen_range(0.0f32, sim_w),
                        macroquad::rand::gen_range(0.0f32, sim_h),
                    ));
                }
            }

            let mut next_gen_bacteria = Vec::new();
            let mut next_gen_predators = Vec::new();
            let mut eaten_food = std::collections::HashSet::new();
            let mut eaten_bacteria = std::collections::HashSet::new();

            // Update bacteria
            for (idx, b) in bacteria.iter_mut().enumerate() {
                b.update(sim_w, sim_h, &food, params.speed_multiplier, &predators);

                // Eat food
                for (i, f) in food.iter().enumerate() {
                    if !eaten_food.contains(&i) {
                         if b.pos.distance(*f) < b.dna.size + 2.0 {
                             b.energy += 30.0;
                             eaten_food.insert(i);
                         }
                    }
                }

                // Check if eaten by predator
                let mut is_eaten = false;
                for p in &predators {
                    if b.pos.distance(p.pos) < p.size + b.dna.size {
                        eaten_bacteria.insert(idx);
                        is_eaten = true;
                        break;
                    }
                }

                // Reproduce (if not eaten)
                if !is_eaten && b.energy > params.reproduction_threshold {
                    b.energy *= 0.5;
                    let offspring = Bacterium {
                        pos: b.pos,
                        vel: -b.vel,
                        dna: b.dna.mutate(&params),
                        energy: b.energy,
                        age: 0.0,
                    };
                    next_gen_bacteria.push(offspring);
                }
            }

            // Update predators
            for p in predators.iter_mut() {
                p.update(sim_w, sim_h, &bacteria, params.speed_multiplier);

                // Eat bacteria
                for (i, b) in bacteria.iter().enumerate() {
                    if !eaten_bacteria.contains(&i) {
                        if p.pos.distance(b.pos) < p.size + b.dna.size {
                            p.energy += 80.0; // Predators gain energy from eating
                            eaten_bacteria.insert(i);
                        }
                    }
                }

                // Reproduce
                if p.energy > params.predator_reproduction_threshold {
                    p.energy *= 0.5;
                    let offspring = Predator {
                        pos: p.pos,
                        vel: -p.vel,
                        energy: p.energy,
                        speed: p.speed,
                        size: p.size,
                        sense_radius: p.sense_radius,
                    };
                    next_gen_predators.push(offspring);
                }
            }

            // Remove eaten food
            let mut new_food = Vec::new();
            for (i, f) in food.iter().enumerate() {
                if !eaten_food.contains(&i) {
                    new_food.push(*f);
                }
            }
            food = new_food;

            // Remove eaten bacteria (iterate backwards to preserve indices)
            let mut indices: Vec<_> = eaten_bacteria.iter().collect();
            indices.sort_by(|a, b| b.cmp(a)); // Sort descending
            for &idx in indices {
                if idx < bacteria.len() {
                    bacteria.swap_remove(idx);
                }
            }

            // Add offspring
            bacteria.append(&mut next_gen_bacteria);
            predators.append(&mut next_gen_predators);

            // Remove dead bacteria and predators
            bacteria.retain(|b| b.energy > 0.0);
            predators.retain(|p| p.energy > 0.0);

            // Fail safe if extinction
            if bacteria.is_empty() {
                 for _ in 0..10 {
                    bacteria.push(Bacterium::new(vec2(
                        macroquad::rand::gen_range(0.0f32, sim_w),
                        macroquad::rand::gen_range(0.0f32, sim_h),
                    ), params.initial_energy));
                 }
            }
        }

        // Calculate stats
        let count = bacteria.len();
        let mut total_speed = 0.0;
        let mut total_size = 0.0;
        for b in &bacteria {
            total_speed += b.dna.speed;
            total_size += b.dna.size;
        }
        let avg_speed = if count > 0 { total_speed / count as f32 } else { 0.0 };
        let avg_size = if count > 0 { total_size / count as f32 } else { 0.0 };
        
        stats.push(count as f32, avg_speed, avg_size, predators.len() as f32);

        // Draw
        clear_background(Color::new(0.03f32, 0.03f32, 0.05f32, 1.0f32));

        // Draw food
        for f in &food {
            draw_circle(f.x, f.y, 2.5, Color::new(0.2f32, 1.0f32, 0.6f32, 0.7f32));
            draw_circle(f.x, f.y, 1.5, Color::new(0.5f32, 1.0f32, 0.8f32, 0.9f32));
        }

        // Draw bacteria
        for b in &bacteria {
            // Glow effect
            draw_circle(b.pos.x, b.pos.y, b.dna.size + 2.0, Color::new(b.dna.color.r, b.dna.color.g, b.dna.color.b, 0.2));
            draw_circle(b.pos.x, b.pos.y, b.dna.size, b.dna.color);
        }

        // Draw predators
        for p in &predators {
            // Glow effect
            draw_circle(p.pos.x, p.pos.y, p.size + 3.0, Color::new(1.0, 0.2, 0.1, 0.3));
            draw_circle(p.pos.x, p.pos.y, p.size, Color::new(0.95, 0.25, 0.15, 0.95));
            draw_circle(p.pos.x, p.pos.y, p.size * 0.6, Color::new(1.0, 0.5, 0.3, 0.8));
            // Eyes
            draw_circle(p.pos.x - 3.0, p.pos.y - 2.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.9));
            draw_circle(p.pos.x + 3.0, p.pos.y - 2.0, 2.0, Color::new(1.0, 1.0, 0.0, 0.9));
        }

        // Draw graphs
        draw_graphs_panel(&stats);

        // Draw UI
        if ui_state.show_ui {
            draw_ui_panel(&mut params, &mut ui_state, count, food.len(), predators.len());
        }

        // Draw FPS
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 25.0, 20.0, Color::new(0.3, 1.0, 0.3, 1.0));

        next_frame().await
    }
}
