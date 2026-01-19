use bevy::prelude::*;

/// Component for the skybox entity
#[derive(Component)]
pub struct Skybox;

/// Component to store the skybox material handle for color updates
#[derive(Component)]
pub struct SkyboxMaterial {
    pub handle: Handle<StandardMaterial>,
}

/// Component for the sun entity
#[derive(Component)]
pub struct Sun;

/// Component for the moon entity
#[derive(Component)]
pub struct Moon;

/// System to spawn the skybox
pub fn spawn_skybox(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create skybox material with realistic sky colors
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.7, 0.9), // Base sky color
        unlit: true, // Sky should not be affected by lighting
        ..default()
    });

    // Create a proper sphere mesh for the skybox
    // Using a high-resolution sphere for smooth appearance
    let sky_mesh = meshes.add(Mesh::try_from(bevy::prelude::Sphere {
        radius: 1000.0,
    }).unwrap());

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_material.clone()),
        SkyboxMaterial { handle: sky_material.clone() },
        Transform::from_translation(Vec3::ZERO),
        Skybox,
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

/// System to update sky color based on time of day
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
    mut sun_query: Query<&mut Transform, With<Sun>>,
    mut moon_query: Query<&mut Transform, With<Moon>>,
) {
    // Calculate sun angle based on time of day
    let sun_angle = time.sun_angle_radians();
    
    // Update sun position
    if let Ok(mut sun_transform) = sun_query.get_single_mut() {
        let sun_position = calculate_celestial_position(sun_angle, 900.0);
        sun_transform.translation = sun_position;
    }
    
    // Calculate moon angle (opposite of sun)
    let moon_angle = time.moon_angle_radians();
    
    // Update moon position
    if let Ok(mut moon_transform) = moon_query.get_single_mut() {
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