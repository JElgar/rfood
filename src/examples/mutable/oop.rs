trait Light {
    fn increase_brightness(&mut self);
    fn decrease_brightness(&mut self);
    fn get_brightness(&self) -> i8;
}

struct Dimmer {
    brightness: i8,
}

struct RGB {
    r: i8,
    g: i8,
    b: i8,
}

impl Light for Dimmer {
    fn increase_brightness(&mut self) {
        self.brightness += 1;
    }

    fn decrease_brightness(&mut self) {
        self.brightness -= 1;
    }

    fn get_brightness(&self) -> i8 {
        return self.brightness;
    }
}

impl Light for RGB {
    fn increase_brightness(&mut self) {
        self.r += 1;
        self.g += 1;
        self.b += 1;
    }

    fn decrease_brightness(&mut self) {
        self.r -= 1;
        self.g -= 1;
        self.b -= 1;
    }

    fn get_brightness(&self) -> i8 {
        return (self.r + self.g + self.b) / 3;
    }
}

pub fn demo() {
    let mut light = Dimmer { brightness: 0 };
    light.increase_brightness();

    println!("Brightness: {}", light.get_brightness());
}
