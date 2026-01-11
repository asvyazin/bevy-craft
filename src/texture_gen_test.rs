// Simple test to verify texture generation module compiles
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_gen_settings() {
        let settings = TextureGenSettings::new();
        assert_eq!(settings.texture_size.x, 256);
        assert_eq!(settings.texture_size.y, 256);
        assert_eq!(settings.noise_scale, 0.05);
        assert_eq!(settings.noise_octaves, 4);
    }

    #[test]
    fn test_noise_to_color() {
        let color = noise_to_color(0.0);
        assert_eq!(color[0], 0); // Red should be 0
        assert_eq!(color[1], 255); // Green should be 255
        
        let color = noise_to_color(1.0);
        assert_eq!(color[0], 255); // Red should be 255
        assert_eq!(color[1], 0); // Green should be 0
    }
}