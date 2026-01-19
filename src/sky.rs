use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::reflect::TypePath;
use encase::ShaderType;

/// Component for the skybox entity
#[derive(Component)]
pub struct Skybox;

/// Component to identify entities with atmospheric scattering materials
#[derive(Component)]
pub struct AtmosphericScatteringEntity {
    pub material_handle: Handle<AtmosphericScatteringMaterial>,
}

/// Component to store the skybox material handle for color updates
#[derive(Component)]
pub struct SkyboxMaterial {
    pub handle: Handle<StandardMaterial>,
}

/// Atmospheric scattering parameters for realistic sky rendering
#[derive(Resource, Clone, Debug)]
pub struct AtmosphericScatteringParams {
    // Rayleigh scattering parameters (molecular scattering - creates blue sky)
    pub rayleigh_coefficient: Vec3,
    pub rayleigh_scale_height: f32,
    
    // Mie scattering parameters (particle scattering - creates haze, sun halos)
    pub mie_coefficient: Vec3,
    pub mie_scale_height: f32,
    pub mie_phase_g: f32,
    
    // Sun parameters
    pub sun_intensity: f32,
    pub sun_angular_radius: f32,
    
    // Atmosphere parameters
    pub planet_radius: f32,
    pub atmosphere_radius: f32,
}

impl Default for AtmosphericScatteringParams {
    fn default() -> Self {
        Self {
            // Rayleigh scattering coefficients (blue > green > red)
            rayleigh_coefficient: Vec3::new(0.002, 0.005, 0.01),
            rayleigh_scale_height: 8000.0,
            
            // Mie scattering coefficients (more uniform)
            mie_coefficient: Vec3::new(0.001, 0.001, 0.001),
            mie_scale_height: 1200.0,
            mie_phase_g: 0.76, // Asymmetry factor for Mie scattering
            
            // Sun parameters
            sun_intensity: 20.0,
            sun_angular_radius: 0.01, // About 0.5 degrees
            
            // Atmosphere parameters (Earth-like)
            planet_radius: 6360000.0, // Earth radius in meters
            atmosphere_radius: 6460000.0, // Atmosphere radius in meters
        }
    }
}

/// Uniform structure for atmospheric scattering shader
#[derive(Clone, ShaderType)]
pub struct AtmosphericScatteringUniform {
    // Rayleigh scattering parameters
    rayleigh_coefficient: Vec3,
    rayleigh_scale_height: f32,
    
    // Mie scattering parameters
    mie_coefficient: Vec3,
    mie_scale_height: f32,
    mie_phase_g: f32,
    
    // Sun parameters
    sun_intensity: f32,
    sun_angular_radius: f32,
    
    // Atmosphere parameters
    planet_radius: f32,
    atmosphere_radius: f32,
    
    // Time of day parameters
    sun_direction: Vec3,
    
    // Camera position
    camera_position: Vec3,
    
    // Padding
    _padding: f32,
}

impl From<&AtmosphericScatteringParams> for AtmosphericScatteringUniform {
    fn from(params: &AtmosphericScatteringParams) -> Self {
        Self {
            rayleigh_coefficient: params.rayleigh_coefficient,
            rayleigh_scale_height: params.rayleigh_scale_height,
            mie_coefficient: params.mie_coefficient,
            mie_scale_height: params.mie_scale_height,
            mie_phase_g: params.mie_phase_g,
            sun_intensity: params.sun_intensity,
            sun_angular_radius: params.sun_angular_radius,
            planet_radius: params.planet_radius,
            atmosphere_radius: params.atmosphere_radius,
            sun_direction: Vec3::ZERO, // Will be updated dynamically
            camera_position: Vec3::ZERO, // Will be updated dynamically
            _padding: 0.0,
        }
    }
}

/// Custom material for atmospheric scattering skybox
#[derive(AsBindGroup, Asset, TypePath, Clone)]
pub struct AtmosphericScatteringMaterial {
    #[uniform(0)]
    pub atmospheric_uniform: AtmosphericScatteringUniform,
}

impl Material for AtmosphericScatteringMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/atmospheric_scattering.wgsl".into()
    }
}

/// Component for the sun entity
#[derive(Component)]
pub struct Sun;

/// Component for the moon entity
#[derive(Component)]
pub struct Moon;

/// System to spawn the skybox
pub fn spawn_skybox(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut atmospheric_materials: ResMut<Assets<AtmosphericScatteringMaterial>>,
    atmospheric_params: Res<AtmosphericScatteringParams>,
) {
    // Create atmospheric scattering material
    let atmospheric_material = atmospheric_materials.add(AtmosphericScatteringMaterial {
        atmospheric_uniform: atmospheric_params.as_ref().into(),
    });

    // Create a proper sphere mesh for the skybox
    // Using a high-resolution sphere for smooth appearance
    let sky_mesh = meshes.add(Mesh::try_from(bevy::prelude::Sphere {
        radius: 1000.0,
    }).unwrap());

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(atmospheric_material.clone()),
        Skybox,
        AtmosphericScatteringEntity {
            material_handle: atmospheric_material,
        },
        Transform::from_translation(Vec3::ZERO),
    ));
}

/// System to spawn sun and moon
pub fn spawn_sun_and_moon(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create sun material (bright yellow)
    let sun_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.8, 0.2),
        emissive: Color::srgb(10.0, 8.0, 2.0).into(), // Make sun glow
        unlit: true,
        ..default()
    });

    // Create moon material (pale blue)
    let moon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9),
        emissive: Color::srgb(1.0, 1.0, 1.4).into(), // Make moon glow softly
        unlit: true,
        ..default()
    });

    // Create sun mesh (larger sphere)
    let sun_mesh = meshes.add(Mesh::try_from(bevy::prelude::Sphere {
        radius: 20.0,
    }).unwrap());

    // Create moon mesh (smaller sphere)
    let moon_mesh = meshes.add(Mesh::try_from(bevy::prelude::Sphere {
        radius: 15.0,
    }).unwrap());

    // Spawn sun at initial position (noon position)
    let sun_angle = 0.0; // Start at noon
    let sun_position = calculate_celestial_position(sun_angle, 900.0);
    commands.spawn((
        Mesh3d(sun_mesh),
        MeshMaterial3d(sun_material),
        Transform::from_translation(sun_position),
        Sun,
    ));

    // Spawn moon at initial position (opposite of sun)
    let moon_angle = sun_angle + std::f32::consts::PI;
    let moon_position = calculate_celestial_position(moon_angle, 900.0);
    commands.spawn((
        Mesh3d(moon_mesh),
        MeshMaterial3d(moon_material),
        Transform::from_translation(moon_position),
        Moon,
    ));
}

/// Calculate celestial body position based on angle and distance
fn calculate_celestial_position(angle: f32, distance: f32) -> Vec3 {
    // Convert angle to position on a sphere
    // angle = 0 is noon (top), angle = PI is midnight (bottom)
    let x = distance * angle.sin();
    let y = distance * angle.cos();
    let z = 0.0; // Keep in the X-Y plane for simplicity
    
    Vec3::new(x, y, z)
}

/// System to update atmospheric scattering parameters based on time of day
pub fn update_atmospheric_scattering(
    time: Res<crate::time::GameTime>,
    mut atmospheric_materials: ResMut<Assets<AtmosphericScatteringMaterial>>,
    query: Query<&AtmosphericScatteringEntity>,
    cameras: Query<&Transform, With<Camera>>,
) {
    // Get camera position for atmospheric scattering calculations
    let camera_position = cameras.single().translation;
    
    // Calculate sun direction based on time of day
    let sun_angle = time.sun_angle_radians();
    let sun_direction = Vec3::new(sun_angle.sin(), sun_angle.cos(), 0.0);
    
    // Update all atmospheric scattering materials
    for entity in &query {
        if let Some(material) = atmospheric_materials.get_mut(&entity.material_handle) {
            // Update sun direction and camera position in the uniform
            material.atmospheric_uniform.sun_direction = sun_direction;
            material.atmospheric_uniform.camera_position = camera_position;
            
            // Adjust scattering parameters based on time of day
            let time_of_day = time.time_of_day_normalized();
            
            // Daytime has more scattering, nighttime has less
            let scattering_intensity = if time.is_day() { 1.0 } else { 0.3 };
            
            material.atmospheric_uniform.rayleigh_coefficient *= scattering_intensity;
            material.atmospheric_uniform.mie_coefficient *= scattering_intensity;
            
            // Adjust sun intensity based on time of day
            let sun_intensity = if time.is_day() { 
                20.0 * (1.0 - (time_of_day - 0.5).abs() * 2.0).max(0.1) // Brighter at noon
            } else {
                2.0 // Dimmer at night (moonlight)
            };
            
            material.atmospheric_uniform.sun_intensity = sun_intensity;
            
            // Log atmospheric scattering parameters for debugging (every 10 seconds)
            if time.current_time % 10.0 < 0.1 {
                println!("🌤️  Atmospheric Scattering Update:");
                println!("   Sun Direction: {:?}", sun_direction);
                println!("   Sun Intensity: {}", sun_intensity);
                println!("   Rayleigh Coefficient: {:?}", material.atmospheric_uniform.rayleigh_coefficient);
                println!("   Mie Coefficient: {:?}", material.atmospheric_uniform.mie_coefficient);
                println!("   Time of Day: {} ({})", time.format_time(), if time.is_day() { "Day" } else { "Night" });
            }
        }
    }
}

/// System to update sky color based on time of day (legacy system, kept for compatibility)
pub fn update_sky_color(
    time: Res<crate::time::GameTime>,
    mut sky_query: Query<&SkyboxMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the sky material and update its color based on time of day
    if let Ok(sky_material) = sky_query.get_single() {
        if let Some(material) = materials.get_mut(&sky_material.handle) {
            let time_of_day = time.time_of_day_normalized();
            
            // Calculate smooth color transitions based on time of day
            let sky_color = calculate_sky_color(time_of_day);
            
            // Apply the new color
            material.base_color = sky_color;
        }
    }
}

/// System to update sun and moon positions based on time of day
pub fn update_sun_and_moon_positions(
    time: Res<crate::time::GameTime>,
    mut transforms: ParamSet<(
        Query<&mut Transform, With<Sun>>,
        Query<&mut Transform, With<Moon>>,
    )>,
) {
    // Calculate sun angle based on time of day
    let sun_angle = time.sun_angle_radians();
    
    // Update sun position
    if let Ok(mut sun_transform) = transforms.p0().get_single_mut() {
        let sun_position = calculate_celestial_position(sun_angle, 900.0);
        sun_transform.translation = sun_position;
    }
    
    // Calculate moon angle (opposite of sun)
    let moon_angle = time.moon_angle_radians();
    
    // Update moon position
    if let Ok(mut moon_transform) = transforms.p1().get_single_mut() {
        let moon_position = calculate_celestial_position(moon_angle, 900.0);
        moon_transform.translation = moon_position;
    }
}

/// Calculate sky color based on time of day (0.0 = midnight, 1.0 = next midnight)
fn calculate_sky_color(time_of_day: f32) -> Color {
    // Define key time points and their corresponding colors
    let dawn_start = 0.25; // 6:00 AM
    let day_start = 0.35;  // 8:00 AM  
    let dusk_start = 0.75; // 6:00 PM
    let night_start = 0.85; // 8:00 PM
    
    // Define colors for different times of day
    let night_color = Color::srgb(0.05, 0.1, 0.2); // Deep blue night
    let dawn_dusk_color = Color::srgb(0.1, 0.3, 0.6); // Pre-dawn/dusk blue
    let day_color = Color::srgb(0.4, 0.7, 0.9); // Bright day sky
    let sunset_color = Color::srgb(0.8, 0.4, 0.2); // Sunset orange
    
    // Calculate smooth transitions between colors
    if time_of_day < dawn_start {
        // Night to dawn transition
        let t = (time_of_day - 0.0) / dawn_start;
        interpolate_color(night_color, dawn_dusk_color, t)
    } else if time_of_day < day_start {
        // Dawn to day transition
        let t = (time_of_day - dawn_start) / (day_start - dawn_start);
        interpolate_color(dawn_dusk_color, day_color, t)
    } else if time_of_day < dusk_start {
        // Daytime
        day_color
    } else if time_of_day < night_start {
        // Day to dusk transition
        let t = (time_of_day - dusk_start) / (night_start - dusk_start);
        interpolate_color(day_color, sunset_color, t)
    } else {
        // Dusk to night transition
        let t = (time_of_day - night_start) / (1.0 - night_start);
        interpolate_color(sunset_color, night_color, t)
    }
}

/// Linear interpolation between two colors
fn interpolate_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    
    // Convert colors to linear RGB for interpolation
    let a_linear = a.to_linear();
    let b_linear = b.to_linear();
    
    // Interpolate each component
    let r = a_linear.red + (b_linear.red - a_linear.red) * t;
    let g = a_linear.green + (b_linear.green - a_linear.green) * t;
    let b = a_linear.blue + (b_linear.blue - a_linear.blue) * t;
    
    // Convert back to sRGB
    Color::linear_rgb(r, g, b)
}