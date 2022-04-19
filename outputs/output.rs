enum Light {
    Dimmer { brightness: i32 },
    RGB { r: i32, g: i32, b: i32 },
}
fn increase_brightness(light: Light) -> Light {
    match &light {
        Light::Dimmer { mut brightness } => {
            brightness += 1;
            Light::Dimmer { brightness }
        }
        Light::RGB {
            mut r,
            mut g,
            mut b,
        } => {
            r += 1;
            g += 1;
            b += 1;
            Light::RGB { r, g, b }
        }
    }
}
fn decrease_brightness(light: Light) -> Light {
    match &light {
        Light::Dimmer { mut brightness } => {
            brightness -= 1;
            Light::Dimmer { brightness }
        }
        Light::RGB {
            mut r,
            mut g,
            mut b,
        } => {
            r -= 1;
            g -= 1;
            b -= 1;
            Light::RGB { r, g, b }
        }
    }
}
fn turn_off(light: Light) -> Light {
    match &light {
        Light::Dimmer { mut brightness } => {
            brightness = 0;
            Light::Dimmer { brightness }
        }
        Light::RGB {
            mut r,
            mut g,
            mut b,
        } => {
            r = 0;
            g = 0;
            b = 0;
            Light::RGB { r, g, b }
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
    println!("Brightness: {}", get_brightness(&light));
}
