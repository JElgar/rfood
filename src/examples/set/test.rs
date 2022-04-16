enum Light {
    Dimmer { brightness: i32 },
    RGB { r: i32, g: i32, b: i32 },
}
fn increase_brightness(light: Light) -> Light {
    match &light {
        Light::Dimmer { brightness } => {
            let mut new = light;
            *brightness += 1;
            new
        }
        Light::RGB { r, g, b } => {
            let mut new = light;
            *r += 1;
            *g += 1;
            *b += 1;
            new
        }
    }
}
fn decrease_brightness(light: Light) -> Light {
    match &light {
        Light::Dimmer { brightness } => {
            let mut new = light;
            *brightness -= 1;
            new
        }
        Light::RGB { r, g, b } => {
            let mut new = light;
            *r -= 1;
            *g -= 1;
            *b -= 1;
            new
        }
    }
}
fn turn_off(light: Light) -> Light {
    match &light {
        Light::Dimmer { brightness } => {
            let mut new = light;
            *brightness = 0;
            new
        }
        Light::RGB { r, g, b } => {
            let mut new = light;
            *r = 0;
            *g = 0;
            *b = 0;
            new
        }
    }
}
fn get_brightness(light: &Light) -> i32 {
    match &light {
        Light::Dimmer { brightness } => {
            return *brightness;
        }
        Light::RGB { r, g, b } => {
            return (*r + *g + *b) / 3;
        }
    }
}
pub fn demo() {
    let mut light = Light::Dimmer { brightness: 0 };
    light = increase_brightness(light);
}
