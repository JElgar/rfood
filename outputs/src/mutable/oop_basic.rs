enum Light {
    Dimmer { brightness: i32 },
}
fn get_brightness(light: &Light) -> i32 {
    match &light {
        Light::Dimmer { brightness } => {
            return *brightness;
        }
    }
}
fn set_brightness(light: Light, brightness: i32) -> Light {
    match &light {
        Light::Dimmer { mut brightness } => {
            brightness = brightness;
            Light::Dimmer { brightness }
        }
    }
}
