// Weather and cloud system for Bevy Craft
// This module handles cloud rendering, weather effects, and atmospheric conditions

#![allow(dead_code)]
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use encase::ShaderType;
use std::f32::consts::PI;

/// Main weather system resource containing all weather-related state
#[derive(Resource, Debug, Clone)]
pub struct WeatherSystem {
    /// Current weather type
    pub current_weather: WeatherType,
    /// Target weather type (for transitions)
    pub target_weather: WeatherType,
    /// Weather transition progress (0.0 to 1.0)
    pub transition_progress: f32,
    /// Weather transition speed
    pub transition_speed: f32,
    /// Cloud coverage (0.0 to 1.0)
    pub cloud_coverage: f32,
    /// Cloud density (0.0 to 1.0)
    pub cloud_density: f32,
    /// Wind direction (normalized vector)
    pub wind_direction: Vec3,
    /// Wind speed
    pub wind_speed: f32,
    /// Precipitation intensity (0.0 to 1.0)
    pub precipitation_intensity: f32,
    /// Temperature (in Celsius)
    pub temperature: f32,
    /// Humidity (0.0 to 1.0)
    pub humidity: f32,
    /// Time since last weather change
    pub time_since_weather_change: f32,
    /// Minimum time between weather changes (seconds)
    pub min_weather_duration: f32,
    /// Maximum time between weather changes (seconds)
    #[allow(dead_code)]
    pub max_weather_duration: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            target_weather: WeatherType::Clear,
            transition_progress: 1.0,
            transition_speed: 0.01,
            cloud_coverage: 0.3,
            cloud_density: 0.5,
            wind_direction: Vec3::new(1.0, 0.0, 0.5).normalize(),
            wind_speed: 2.0,
            precipitation_intensity: 0.0,
            temperature: 20.0,
            humidity: 0.5,
            time_since_weather_change: 0.0,
            min_weather_duration: 300.0,  // 5 minutes
            max_weather_duration: 1800.0, // 30 minutes
        }
    }
}

/// Component to identify cloud entities
#[derive(Component)]
pub struct CloudEntity;

/// Component to identify weather particle entities (rain, snow, etc.)
#[derive(Component)]
pub struct WeatherParticleEntity;

/// Component to identify weather effect entities
#[derive(Component)]
#[allow(dead_code)]
pub struct WeatherEffectEntity;

/// Weather types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WeatherType {
    Clear,        // No clouds, sunny
    PartlyCloudy, // Some clouds
    Cloudy,       // Mostly cloudy
    Overcast,     // Completely overcast
    Rain,         // Light to moderate rain
    HeavyRain,    // Heavy rain
    Thunderstorm, // Rain with thunder and lightning
    Snow,         // Light to moderate snow
    HeavySnow,    // Heavy snow
    Fog,          // Dense fog
    Sandstorm,    // Sandstorm (for desert biomes)
}

impl WeatherType {
    /// Get the display name for the weather type
    pub fn display_name(&self) -> &str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::PartlyCloudy => "Partly Cloudy",
            WeatherType::Cloudy => "Cloudy",
            WeatherType::Overcast => "Overcast",
            WeatherType::Rain => "Rain",
            WeatherType::HeavyRain => "Heavy Rain",
            WeatherType::Thunderstorm => "Thunderstorm",
            WeatherType::Snow => "Snow",
            WeatherType::HeavySnow => "Heavy Snow",
            WeatherType::Fog => "Fog",
            WeatherType::Sandstorm => "Sandstorm",
        }
    }

    /// Check if this weather type has precipitation
    #[allow(dead_code)]
    pub fn has_precipitation(&self) -> bool {
        matches!(
            self,
            WeatherType::Rain
                | WeatherType::HeavyRain
                | WeatherType::Thunderstorm
                | WeatherType::Snow
                | WeatherType::HeavySnow
        )
    }

    /// Check if this weather type is a storm
    pub fn is_storm(&self) -> bool {
        matches!(
            self,
            WeatherType::Thunderstorm | WeatherType::HeavyRain | WeatherType::HeavySnow
        )
    }

    /// Get typical cloud coverage for this weather type
    pub fn typical_cloud_coverage(&self) -> f32 {
        match self {
            WeatherType::Clear => 0.1,
            WeatherType::PartlyCloudy => 0.4,
            WeatherType::Cloudy => 0.7,
            WeatherType::Overcast => 0.9,
            WeatherType::Rain => 0.8,
            WeatherType::HeavyRain => 0.9,
            WeatherType::Thunderstorm => 0.95,
            WeatherType::Snow => 0.8,
            WeatherType::HeavySnow => 0.9,
            WeatherType::Fog => 0.7,
            WeatherType::Sandstorm => 0.6,
        }
    }

    /// Get typical cloud density for this weather type
    pub fn typical_cloud_density(&self) -> f32 {
        match self {
            WeatherType::Clear => 0.1,
            WeatherType::PartlyCloudy => 0.3,
            WeatherType::Cloudy => 0.6,
            WeatherType::Overcast => 0.8,
            WeatherType::Rain => 0.7,
            WeatherType::HeavyRain => 0.9,
            WeatherType::Thunderstorm => 0.9,
            WeatherType::Snow => 0.7,
            WeatherType::HeavySnow => 0.8,
            WeatherType::Fog => 0.5,
            WeatherType::Sandstorm => 0.4,
        }
    }
}

/// Cloud layer configuration
#[derive(Debug, Clone)]
pub struct CloudLayer {
    /// Altitude of the cloud layer
    pub altitude: f32,
    /// Scale of the cloud layer
    pub scale: f32,
    /// Density of the cloud layer
    pub density: f32,
    /// Speed of the cloud layer
    pub speed: f32,
    /// Direction of the cloud layer
    pub direction: Vec2,
    /// Noise octaves for cloud generation
    #[allow(dead_code)]
    pub noise_octaves: u32,
    /// Noise frequency for cloud generation
    #[allow(dead_code)]
    pub noise_frequency: f32,
    /// Noise persistence for cloud generation
    #[allow(dead_code)]
    pub noise_persistence: f32,
}

impl Default for CloudLayer {
    fn default() -> Self {
        Self {
            altitude: 200.0,
            scale: 1000.0,
            density: 0.5,
            speed: 0.01,
            direction: Vec2::new(1.0, 0.5).normalize(),
            noise_octaves: 4,
            noise_frequency: 0.001,
            noise_persistence: 0.5,
        }
    }
}

/// Cloud system resource
#[derive(Resource, Debug, Clone)]
pub struct CloudSystem {
    /// Multiple cloud layers for realistic cloud rendering
    pub layers: Vec<CloudLayer>,
    /// Global cloud coverage multiplier
    #[allow(dead_code)]
    pub coverage_multiplier: f32,
    /// Global cloud density multiplier
    #[allow(dead_code)]
    pub density_multiplier: f32,
    /// Cloud animation time
    pub animation_time: f32,
    /// Cloud animation speed
    pub animation_speed: f32,
}

impl Default for CloudSystem {
    fn default() -> Self {
        Self {
            layers: vec![
                CloudLayer {
                    altitude: 150.0,
                    scale: 800.0,
                    density: 0.4,
                    speed: 0.005,
                    direction: Vec2::new(1.0, 0.2).normalize(),
                    noise_octaves: 3,
                    noise_frequency: 0.0008,
                    noise_persistence: 0.6,
                },
                CloudLayer {
                    altitude: 250.0,
                    scale: 1200.0,
                    density: 0.6,
                    speed: 0.008,
                    direction: Vec2::new(1.0, 0.3).normalize(),
                    noise_octaves: 4,
                    noise_frequency: 0.0012,
                    noise_persistence: 0.5,
                },
                CloudLayer {
                    altitude: 400.0,
                    scale: 2000.0,
                    density: 0.3,
                    speed: 0.015,
                    direction: Vec2::new(1.0, 0.1).normalize(),
                    noise_octaves: 2,
                    noise_frequency: 0.0005,
                    noise_persistence: 0.7,
                },
            ],
            coverage_multiplier: 1.0,
            density_multiplier: 1.0,
            animation_time: 0.0,
            animation_speed: 0.001,
        }
    }
}

/// Weather particle materials resource
#[derive(Resource, Debug, Clone)]
pub struct WeatherParticleMaterials {
    pub rain_material: Handle<StandardMaterial>,
    pub snow_material: Handle<StandardMaterial>,
}

/// Weather effects configuration
#[derive(Resource, Debug, Clone)]
pub struct WeatherEffects {
    /// Rain particle count
    pub rain_particle_count: u32,
    /// Snow particle count
    pub snow_particle_count: u32,
    /// Rain particle speed
    pub rain_particle_speed: f32,
    /// Snow particle speed
    pub snow_particle_speed: f32,
    /// Rain particle size
    pub rain_particle_size: f32,
    /// Snow particle size
    pub snow_particle_size: f32,
    /// Lightning flash intensity
    #[allow(dead_code)]
    pub lightning_flash_intensity: f32,
    /// Lightning flash duration
    #[allow(dead_code)]
    pub lightning_flash_duration: f32,
    /// Time between lightning strikes
    pub lightning_frequency: f32,
    /// Time since last lightning strike
    pub time_since_last_lightning: f32,
}

impl Default for WeatherEffects {
    fn default() -> Self {
        Self {
            rain_particle_count: 1000,
            snow_particle_count: 1500,
            rain_particle_speed: 15.0,
            snow_particle_speed: 2.0,
            rain_particle_size: 0.5,
            snow_particle_size: 0.3,
            lightning_flash_intensity: 3.0,
            lightning_flash_duration: 0.2,
            lightning_frequency: 10.0,
            time_since_last_lightning: 0.0,
        }
    }
}

/// Uniform structure for cloud shader
#[derive(Clone, ShaderType)]
#[allow(dead_code)]
pub struct CloudUniform {
    // Cloud layer parameters using Vec4 for guaranteed 16-byte alignment
    layer_altitudes: Vec4,
    layer_scales: Vec4,
    layer_densities: Vec4,
    layer_speeds: Vec4,

    // Layer directions as Vec4 for proper alignment
    layer_directions: Vec4, // xyzw where x=layer0_x, y=layer0_y, z=layer1_x, w=layer1_y
    layer_directions2: Vec4, // xyzw where x=layer2_x, y=layer2_y, z=layer3_x, w=layer3_y

    // Global cloud parameters
    global_coverage: f32,
    global_density: f32,
    animation_time: f32,
    animation_speed: f32,

    // Camera and lighting parameters
    camera_position: Vec4, // Vec3 + padding
    sun_direction: Vec4,   // Vec3 + padding

    // Weather parameters
    weather_type: u32,
    precipitation_intensity: f32,
    wind_direction: Vec4, // Vec3 + padding
    wind_speed: f32,
}

impl Default for CloudUniform {
    fn default() -> Self {
        Self {
            layer_altitudes: Vec4::ZERO,
            layer_scales: Vec4::ZERO,
            layer_densities: Vec4::ZERO,
            layer_speeds: Vec4::ZERO,
            layer_directions: Vec4::ZERO,
            layer_directions2: Vec4::ZERO,
            global_coverage: 0.5,
            global_density: 0.5,
            animation_time: 0.0,
            animation_speed: 0.001,
            camera_position: Vec4::new(0.0, 0.0, 0.0, 0.0),
            sun_direction: Vec4::new(0.0, 1.0, 0.0, 0.0),
            weather_type: 0,
            precipitation_intensity: 0.0,
            wind_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
            wind_speed: 1.0,
        }
    }
}

/// Custom material for cloud rendering
#[derive(AsBindGroup, Asset, TypePath, Clone)]
pub struct CloudMaterial {
    #[uniform(0)]
    pub cloud_uniform: CloudUniform,
}

impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cloud.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/cloud_vertex.wgsl".into()
    }
}

/// System to initialize the weather system
pub fn initialize_weather_system(mut commands: Commands) {
    commands.insert_resource(WeatherSystem::default());
    commands.insert_resource(CloudSystem::default());
    commands.insert_resource(WeatherEffects::default());
}

/// System to spawn cloud layers
/// Note: This is a startup system that creates basic cloud layers
/// Actual cloud rendering parameters are updated dynamically
pub fn spawn_cloud_layers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    // Create a large plane mesh for cloud rendering
    let cloud_mesh = meshes.add(
        Mesh::try_from(bevy::prelude::Plane3d {
            normal: bevy::math::Dir3::Y,
            half_size: Vec2::new(2500.0, 2500.0),
            ..default()
        })
        .unwrap(),
    );

    // Create cloud material with default parameters
    let cloud_uniform = CloudUniform::default();
    let cloud_material = materials.add(CloudMaterial { cloud_uniform });

    // Spawn cloud layers at default altitudes
    // These will be updated dynamically by the cloud rendering system
    let default_altitudes = [150.0, 250.0, 400.0];

    for altitude in default_altitudes.iter() {
        commands.spawn((
            Mesh3d(cloud_mesh.clone()),
            MeshMaterial3d(cloud_material.clone()),
            CloudEntity,
            Transform::from_translation(Vec3::new(0.0, *altitude, 0.0))
                .with_rotation(Quat::from_rotation_x(-PI / 2.0)), // Make plane horizontal
        ));
    }
}

/// System to update weather conditions based on time and random factors
pub fn update_weather_system(
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
    mut weather_system: ResMut<WeatherSystem>,
) {
    // Update weather transition
    if weather_system.transition_progress < 1.0 {
        weather_system.transition_progress += weather_system.transition_speed * time.delta_secs();
        weather_system.transition_progress = weather_system.transition_progress.min(1.0);

        // Interpolate between current and target weather
        let current_weather = weather_system.current_weather;
        let target_weather = weather_system.target_weather;

        // Interpolate cloud coverage and density
        let current_coverage = current_weather.typical_cloud_coverage();
        let target_coverage = target_weather.typical_cloud_coverage();
        let current_density = current_weather.typical_cloud_density();
        let target_density = target_weather.typical_cloud_density();

        weather_system.cloud_coverage = current_coverage
            + (target_coverage - current_coverage) * weather_system.transition_progress;
        weather_system.cloud_density = current_density
            + (target_density - current_density) * weather_system.transition_progress;

        // Update precipitation intensity based on weather type
        weather_system.precipitation_intensity = match target_weather {
            WeatherType::Rain => 0.5,
            WeatherType::HeavyRain => 0.9,
            WeatherType::Thunderstorm => 0.8,
            WeatherType::Snow => 0.4,
            WeatherType::HeavySnow => 0.7,
            _ => 0.0,
        } * weather_system.transition_progress;
    }

    // Update time since last weather change
    weather_system.time_since_weather_change += time.delta_secs();

    // Randomly change weather after minimum duration
    if weather_system.time_since_weather_change > weather_system.min_weather_duration
        && weather_system.transition_progress >= 1.0
        && !weather_system.target_weather.is_storm()
    // Don't change away from storms too quickly
    {
        // Check if we should change weather (random chance based on time)
        let weather_change_chance = 0.001 * time.delta_secs(); // Small chance per frame
        if rand::random::<f32>() < weather_change_chance {
            change_weather_randomly(&mut weather_system);
        }
    }

    // Update wind based on weather
    update_wind_for_weather(&mut weather_system);

    // Update temperature and humidity based on time of day
    update_environmental_conditions(&game_time, &mut weather_system);
}

/// Change weather to a random appropriate type
fn change_weather_randomly(weather_system: &mut WeatherSystem) {
    let current_weather = weather_system.current_weather;

    // Define possible weather transitions
    let possible_weather_types = match current_weather {
        WeatherType::Clear => vec![
            WeatherType::Clear,        // 30% chance to stay clear
            WeatherType::PartlyCloudy, // 50% chance to get some clouds
            WeatherType::Cloudy,       // 20% chance to get cloudy
        ],
        WeatherType::PartlyCloudy => vec![
            WeatherType::Clear,        // 20% chance to clear up
            WeatherType::PartlyCloudy, // 40% chance to stay partly cloudy
            WeatherType::Cloudy,       // 30% chance to get more cloudy
            WeatherType::Overcast,     // 10% chance to get overcast
        ],
        WeatherType::Cloudy => vec![
            WeatherType::PartlyCloudy, // 20% chance to clear up
            WeatherType::Cloudy,       // 30% chance to stay cloudy
            WeatherType::Overcast,     // 30% chance to get overcast
            WeatherType::Rain,         // 20% chance to start raining
        ],
        WeatherType::Overcast => vec![
            WeatherType::Cloudy,       // 20% chance to clear up
            WeatherType::Overcast,     // 30% chance to stay overcast
            WeatherType::Rain,         // 30% chance to start raining
            WeatherType::HeavyRain,    // 10% chance to get heavy rain
            WeatherType::Thunderstorm, // 10% chance to get thunderstorm
        ],
        WeatherType::Rain => vec![
            WeatherType::Cloudy,       // 20% chance to clear up
            WeatherType::Overcast,     // 30% chance to stay overcast
            WeatherType::Rain,         // 30% chance to continue raining
            WeatherType::HeavyRain,    // 10% chance to get heavier
            WeatherType::Thunderstorm, // 10% chance to get thunderstorm
        ],
        WeatherType::HeavyRain => vec![
            WeatherType::Overcast,     // 20% chance to clear up
            WeatherType::Rain,         // 40% chance to lighten up
            WeatherType::HeavyRain,    // 30% chance to continue heavy rain
            WeatherType::Thunderstorm, // 10% chance to get thunderstorm
        ],
        WeatherType::Thunderstorm => vec![
            WeatherType::Overcast,     // 10% chance to clear up
            WeatherType::Rain,         // 30% chance to lighten up
            WeatherType::HeavyRain,    // 40% chance to continue heavy rain
            WeatherType::Thunderstorm, // 20% chance to continue thunderstorm
        ],
        WeatherType::Snow => vec![
            WeatherType::Cloudy,    // 20% chance to clear up
            WeatherType::Overcast,  // 30% chance to stay overcast
            WeatherType::Snow,      // 30% chance to continue snowing
            WeatherType::HeavySnow, // 20% chance to get heavier
        ],
        WeatherType::HeavySnow => vec![
            WeatherType::Overcast,  // 20% chance to clear up
            WeatherType::Snow,      // 40% chance to lighten up
            WeatherType::HeavySnow, // 40% chance to continue heavy snow
        ],
        WeatherType::Fog => vec![
            WeatherType::Clear,        // 20% chance to clear up
            WeatherType::PartlyCloudy, // 30% chance to get some clouds
            WeatherType::Fog,          // 50% chance to stay foggy
        ],
        WeatherType::Sandstorm => vec![
            WeatherType::Clear,        // 30% chance to clear up
            WeatherType::PartlyCloudy, // 40% chance to get some clouds
            WeatherType::Sandstorm,    // 30% chance to continue sandstorm
        ],
    };

    // Choose a random weather type from possible transitions
    let new_weather =
        possible_weather_types[rand::random::<usize>() % possible_weather_types.len()];

    // Start transition to new weather
    weather_system.current_weather = current_weather;
    weather_system.target_weather = new_weather;
    weather_system.transition_progress = 0.0;
    weather_system.time_since_weather_change = 0.0;

    println!(
        "🌦️  Weather changing from {} to {}",
        current_weather.display_name(),
        new_weather.display_name()
    );
}

/// Update wind conditions based on current weather
fn update_wind_for_weather(weather_system: &mut WeatherSystem) {
    // Base wind direction with some random variation
    let base_wind = Vec3::new(1.0, 0.0, 0.2).normalize();
    let wind_variation = Vec3::new(
        (rand::random::<f32>() - 0.5) * 0.1,
        0.0,
        (rand::random::<f32>() - 0.5) * 0.1,
    );

    weather_system.wind_direction = (base_wind + wind_variation).normalize();

    // Adjust wind speed based on weather
    weather_system.wind_speed = match weather_system.target_weather {
        WeatherType::Clear => 1.0,
        WeatherType::PartlyCloudy => 1.5,
        WeatherType::Cloudy => 2.0,
        WeatherType::Overcast => 2.5,
        WeatherType::Rain => 3.0,
        WeatherType::HeavyRain => 4.0,
        WeatherType::Thunderstorm => 5.0,
        WeatherType::Snow => 2.0,
        WeatherType::HeavySnow => 3.0,
        WeatherType::Fog => 0.5,
        WeatherType::Sandstorm => 6.0,
    } + (rand::random::<f32>() - 0.5) * 0.5; // Add some randomness
}

/// Update environmental conditions based on time of day
fn update_environmental_conditions(
    game_time: &crate::time::GameTime,
    weather_system: &mut WeatherSystem,
) {
    let time_of_day = game_time.time_of_day_normalized();

    // Temperature varies with time of day (colder at night, warmer during day)
    let base_temperature = 20.0; // Base temperature in Celsius
    let temperature_variation = 10.0 * (time_of_day * 2.0 * PI).sin(); // Sinusoidal variation

    weather_system.temperature = base_temperature + temperature_variation;

    // Humidity varies with time of day and weather
    let base_humidity = 0.5;
    let humidity_variation = 0.2 * (1.0 - (time_of_day - 0.5).abs() * 2.0); // Higher humidity around dawn/dusk

    weather_system.humidity = (base_humidity + humidity_variation).clamp(0.2, 0.9);

    // Adjust for specific weather types
    match weather_system.target_weather {
        WeatherType::Rain | WeatherType::HeavyRain | WeatherType::Thunderstorm => {
            weather_system.humidity = (weather_system.humidity + 0.3).min(1.0);
        }
        WeatherType::Snow | WeatherType::HeavySnow => {
            weather_system.temperature -= 15.0;
            weather_system.humidity = (weather_system.humidity + 0.2).min(1.0);
        }
        WeatherType::Fog => {
            weather_system.humidity = (weather_system.humidity + 0.4).min(1.0);
        }
        WeatherType::Sandstorm => {
            weather_system.temperature += 10.0;
            weather_system.humidity = (weather_system.humidity - 0.3).max(0.1);
        }
        _ => {}
    }
}

/// System to spawn cloud layers
pub fn update_cloud_rendering(
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
    weather_system: Res<WeatherSystem>,
    cloud_system: Res<CloudSystem>,
    cameras: Query<&Transform, With<Camera>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
    _cloud_query: Query<&CloudEntity>,
) {
    // Update cloud animation time
    let mut updated_cloud_system = cloud_system.clone();
    updated_cloud_system.animation_time += time.delta_secs() * updated_cloud_system.animation_speed;

    // Get camera position
    let camera_position = cameras.single().translation;

    // Calculate sun direction
    let sun_angle = game_time.sun_angle_radians();
    let sun_direction = Vec3::new(sun_angle.sin(), sun_angle.cos(), 0.0);

    // Update cloud uniform with current weather and system data
    let mut cloud_uniform = CloudUniform::default();

    // Set up cloud layers from the cloud system using Vec4 packing
    let mut altitudes = Vec4::ZERO;
    let mut scales = Vec4::ZERO;
    let mut densities = Vec4::ZERO;
    let mut speeds = Vec4::ZERO;
    let mut directions = Vec4::ZERO;
    let mut directions2 = Vec4::ZERO;

    for (i, layer) in updated_cloud_system.layers.iter().enumerate() {
        if i < 4 {
            // Set individual components
            match i {
                0 => {
                    altitudes.x = layer.altitude;
                    scales.x = layer.scale;
                    densities.x = layer.density;
                    speeds.x = layer.speed;
                    directions.x = layer.direction.x;
                    directions.y = layer.direction.y;
                }
                1 => {
                    altitudes.y = layer.altitude;
                    scales.y = layer.scale;
                    densities.y = layer.density;
                    speeds.y = layer.speed;
                    directions.z = layer.direction.x;
                    directions.w = layer.direction.y;
                }
                2 => {
                    altitudes.z = layer.altitude;
                    scales.z = layer.scale;
                    densities.z = layer.density;
                    speeds.z = layer.speed;
                    directions2.x = layer.direction.x;
                    directions2.y = layer.direction.y;
                }
                3 => {
                    altitudes.w = layer.altitude;
                    scales.w = layer.scale;
                    densities.w = layer.density;
                    speeds.w = layer.speed;
                    directions2.z = layer.direction.x;
                    directions2.w = layer.direction.y;
                }
                _ => {}
            }
        }
    }

    cloud_uniform.layer_altitudes = altitudes;
    cloud_uniform.layer_scales = scales;
    cloud_uniform.layer_densities = densities;
    cloud_uniform.layer_speeds = speeds;
    cloud_uniform.layer_directions = directions;
    cloud_uniform.layer_directions2 = directions2;

    // Apply global weather effects
    cloud_uniform.global_coverage = weather_system.cloud_coverage;
    cloud_uniform.global_density = weather_system.cloud_density;
    cloud_uniform.animation_time = updated_cloud_system.animation_time;
    cloud_uniform.animation_speed = updated_cloud_system.animation_speed;
    cloud_uniform.camera_position = camera_position.extend(0.0); // Convert Vec3 to Vec4
    cloud_uniform.sun_direction = sun_direction.extend(0.0); // Convert Vec3 to Vec4
    cloud_uniform.weather_type = weather_system.target_weather as u32;
    cloud_uniform.precipitation_intensity = weather_system.precipitation_intensity;
    cloud_uniform.wind_direction = weather_system.wind_direction.extend(0.0); // Convert Vec3 to Vec4
    cloud_uniform.wind_speed = weather_system.wind_speed;

    // Update all cloud materials
    for (_, material) in materials.iter_mut() {
        material.cloud_uniform = cloud_uniform.clone();
    }

    // Update the cloud system with new parameters
    // This would normally be done through a different mechanism in a real implementation
}

/// System to spawn weather particles (rain, snow, etc.)
/// Note: This is a startup system that just initializes particle systems
/// Actual particle spawning happens in the update phase
pub fn spawn_weather_particles(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // During startup, we just prepare the particle systems
    // Actual particle spawning will happen in the update phase based on current weather

    // Create a placeholder rain material for later use
    let rain_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.8, 1.0),
        emissive: Color::srgb(0.1, 0.2, 0.3).into(),
        unlit: true,
        ..default()
    });

    // Create a placeholder snow material for later use
    let snow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        emissive: Color::srgb(0.8, 0.9, 1.0).into(),
        unlit: true,
        ..default()
    });

    // Store these materials in a resource for later use
    commands.insert_resource(WeatherParticleMaterials {
        rain_material,
        snow_material,
    });
}

/// Spawn rain particles (dynamic version for update phase)
fn spawn_rain_particles_dynamic(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    rain_material: &Handle<StandardMaterial>,
    _weather_system: &WeatherSystem,
    weather_effects: &WeatherEffects,
) {
    let particle_count = weather_effects.rain_particle_count;
    let particle_size = weather_effects.rain_particle_size;
    let spawn_area = 1000.0; // Area where particles spawn

    // Create rain particle mesh (simple cylinder)
    let rain_mesh = meshes.add(
        Mesh::try_from(bevy::prelude::Cylinder {
            radius: particle_size * 0.1,
            half_height: particle_size * 0.5,
            ..default()
        })
        .unwrap(),
    );

    // Spawn rain particles
    for _i in 0..particle_count {
        let x = (rand::random::<f32>() - 0.5) * spawn_area;
        let z = (rand::random::<f32>() - 0.5) * spawn_area;
        let y = 300.0 + rand::random::<f32>() * 100.0;

        commands.spawn((
            Mesh3d(rain_mesh.clone()),
            MeshMaterial3d(rain_material.clone()),
            WeatherParticleEntity,
            Transform::from_translation(Vec3::new(x, y, z)),
        ));
    }
}

/// Spawn snow particles (dynamic version for update phase)
fn spawn_snow_particles_dynamic(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    snow_material: &Handle<StandardMaterial>,
    _weather_system: &WeatherSystem,
    weather_effects: &WeatherEffects,
) {
    let particle_count = weather_effects.snow_particle_count;
    let particle_size = weather_effects.snow_particle_size;
    let spawn_area = 1000.0; // Area where particles spawn

    // Create snow particle mesh (simple sphere)
    let snow_mesh = meshes.add(
        Mesh::try_from(bevy::prelude::Sphere {
            radius: particle_size,
            ..default()
        })
        .unwrap(),
    );

    // Spawn snow particles
    for _i in 0..particle_count {
        let x = (rand::random::<f32>() - 0.5) * spawn_area;
        let z = (rand::random::<f32>() - 0.5) * spawn_area;
        let y = 300.0 + rand::random::<f32>() * 100.0;

        commands.spawn((
            Mesh3d(snow_mesh.clone()),
            MeshMaterial3d(snow_material.clone()),
            WeatherParticleEntity,
            Transform::from_translation(Vec3::new(x, y, z)),
        ));
    }
}

/// System to spawn weather particles during update phase based on current weather
pub fn spawn_weather_particles_dynamic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    particle_materials: Res<WeatherParticleMaterials>,
    weather_system: Res<WeatherSystem>,
    weather_effects: Res<WeatherEffects>,
    particle_query: Query<Entity, With<WeatherParticleEntity>>,
) {
    // First, despawn existing particles to avoid accumulation
    for entity in &particle_query {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            // Double-check entity exists before despawning
            if entity_commands.id() == entity {
                entity_commands.despawn();
            }
        }
    }

    // Spawn particles based on current weather
    match weather_system.target_weather {
        WeatherType::Rain | WeatherType::HeavyRain | WeatherType::Thunderstorm => {
            spawn_rain_particles_dynamic(
                &mut commands,
                &mut meshes,
                &particle_materials.rain_material,
                &weather_system,
                &weather_effects,
            );
        }
        WeatherType::Snow | WeatherType::HeavySnow => {
            spawn_snow_particles_dynamic(
                &mut commands,
                &mut meshes,
                &particle_materials.snow_material,
                &weather_system,
                &weather_effects,
            );
        }
        _ => {
            // No particles for other weather types
        }
    }
}

/// System to update weather particles (animation, movement, etc.)
pub fn update_weather_particles(
    time: Res<Time>,
    weather_system: Res<WeatherSystem>,
    weather_effects: Res<WeatherEffects>,
    mut particle_query: Query<&mut Transform, With<WeatherParticleEntity>>,
) {
    // Update particles based on weather type
    match weather_system.target_weather {
        WeatherType::Rain | WeatherType::HeavyRain | WeatherType::Thunderstorm => {
            update_rain_particles(
                &time,
                &weather_system,
                &weather_effects,
                &mut particle_query,
            );
        }
        WeatherType::Snow | WeatherType::HeavySnow => {
            update_snow_particles(
                &time,
                &weather_system,
                &weather_effects,
                &mut particle_query,
            );
        }
        _ => {
            // No update for other weather types
        }
    }
}

/// Update rain particles
fn update_rain_particles(
    time: &Time,
    weather_system: &WeatherSystem,
    weather_effects: &WeatherEffects,
    particle_query: &mut Query<&mut Transform, With<WeatherParticleEntity>>,
) {
    let fall_speed = weather_effects.rain_particle_speed;
    let wind_influence = weather_system.wind_speed * 0.1;

    for mut transform in particle_query.iter_mut() {
        // Apply gravity
        transform.translation.y -= fall_speed * time.delta_secs();

        // Apply wind
        transform.translation.x +=
            weather_system.wind_direction.x * wind_influence * time.delta_secs();
        transform.translation.z +=
            weather_system.wind_direction.z * wind_influence * time.delta_secs();

        // Reset particle if it falls below ground
        if transform.translation.y < 0.0 {
            transform.translation.y = 300.0 + rand::random::<f32>() * 100.0;
            transform.translation.x = (rand::random::<f32>() - 0.5) * 1000.0;
            transform.translation.z = (rand::random::<f32>() - 0.5) * 1000.0;
        }
    }
}

/// Update snow particles
fn update_snow_particles(
    time: &Time,
    weather_system: &WeatherSystem,
    weather_effects: &WeatherEffects,
    particle_query: &mut Query<&mut Transform, With<WeatherParticleEntity>>,
) {
    let fall_speed = weather_effects.snow_particle_speed;
    let wind_influence = weather_system.wind_speed * 0.2;
    let sway_amount = 0.5;

    for mut transform in particle_query.iter_mut() {
        // Apply gravity with some randomness
        transform.translation.y -=
            fall_speed * time.delta_secs() * (0.8 + rand::random::<f32>() * 0.4);

        // Apply wind with swaying motion
        transform.translation.x +=
            weather_system.wind_direction.x * wind_influence * time.delta_secs();
        transform.translation.z +=
            weather_system.wind_direction.z * wind_influence * time.delta_secs();

        // Add some swaying motion
        transform.translation.x += (rand::random::<f32>() - 0.5) * sway_amount * time.delta_secs();
        transform.translation.z += (rand::random::<f32>() - 0.5) * sway_amount * time.delta_secs();

        // Reset particle if it falls below ground
        if transform.translation.y < 0.0 {
            transform.translation.y = 300.0 + rand::random::<f32>() * 100.0;
            transform.translation.x = (rand::random::<f32>() - 0.5) * 1000.0;
            transform.translation.z = (rand::random::<f32>() - 0.5) * 1000.0;
        }
    }
}

/// System to handle lightning effects for thunderstorms
pub fn update_lightning_effects(
    time: Res<Time>,
    mut weather_effects: ResMut<WeatherEffects>,
    weather_system: Res<WeatherSystem>,
) {
    // Only process lightning for thunderstorms
    if weather_system.target_weather != WeatherType::Thunderstorm {
        return;
    }

    // Update time since last lightning
    weather_effects.time_since_last_lightning += time.delta_secs();

    // Check if it's time for lightning
    if weather_effects.time_since_last_lightning >= weather_effects.lightning_frequency {
        // Random chance to trigger lightning
        if rand::random::<f32>() < 0.3 {
            // 30% chance per interval
            trigger_lightning(&mut weather_effects);
        }
    }
}

/// Trigger a lightning strike
fn trigger_lightning(weather_effects: &mut WeatherEffects) {
    println!("⚡ Lightning strike!");
    weather_effects.time_since_last_lightning = 0.0;

    // In a real implementation, this would:
    // 1. Create a bright flash effect
    // 2. Spawn lightning bolt particles
    // 3. Play thunder sound
    // 4. Potentially start fires or affect gameplay
}

/// System to display weather information (for debugging)
pub fn display_weather_info(
    weather_system: Res<WeatherSystem>,
    game_time: Res<crate::time::GameTime>,
) {
    if game_time.current_time % 5.0 < 0.1 {
        // Display every 5 seconds
        println!("🌦️  Weather Info:");
        println!(
            "   Current: {}",
            weather_system.current_weather.display_name()
        );
        println!(
            "   Target: {}",
            weather_system.target_weather.display_name()
        );
        println!(
            "   Transition: {:.1}%",
            weather_system.transition_progress * 100.0
        );
        println!(
            "   Cloud Coverage: {:.1}%",
            weather_system.cloud_coverage * 100.0
        );
        println!(
            "   Cloud Density: {:.1}%",
            weather_system.cloud_density * 100.0
        );
        println!(
            "   Precipitation: {:.1}%",
            weather_system.precipitation_intensity * 100.0
        );
        println!("   Temperature: {:.1}°C", weather_system.temperature);
        println!("   Humidity: {:.1}%", weather_system.humidity * 100.0);
        println!(
            "   Wind: {:.1} m/s {:?}",
            weather_system.wind_speed, weather_system.wind_direction
        );
        println!(
            "   Time since change: {:.1}s",
            weather_system.time_since_weather_change
        );
    }
}
