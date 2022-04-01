enum DimmableLight {
    Dimmer(i8),
    RGB(i8, i8, i8),
}

fn increase_brightness(light: DimmableLight) -> DimmableLight {
    return match light {
        DimmableLight::Dimmer(b) => DimmableLight::Dimmer(b + 1),
        DimmableLight::RGB(r, g, b) => DimmableLight::RGB(r + 1, g + 1, b + 1),
    }
}

fn decrease_brightness(light: DimmableLight) -> DimmableLight {
    return match light {
        DimmableLight::Dimmer(b) => DimmableLight::Dimmer(b - 1),
        DimmableLight::RGB(r, g, b) => DimmableLight::RGB(r - 1, g - 1, b - 1),
    }
}

fn get_brightness(light: DimmableLight) -> i8 {
    return match light {
        DimmableLight::Dimmer(b) => b,
        DimmableLight::RGB(r, g, b) => (r + g + b) / 3,
    }
}

pub fn dimmer() {
    let mut dimmer = DimmableLight::Dimmer(10);
    dimmer = increase_brightness(dimmer);

    println!("Brightness: {}", get_brightness(dimmer));
}
