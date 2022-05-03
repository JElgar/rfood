pub trait Light {
    fn increase_brightness(&mut self);
    fn decrease_brightness(&mut self);
    fn turn_off(&mut self);

    fn get_brightness(&self) -> i32;
}

pub struct Dimmer {
    pub brightness: i32,
}

pub struct RGB {
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

impl Light for Dimmer {
    fn increase_brightness(&mut self) {
        self.brightness += 1;
    }

    fn decrease_brightness(&mut self) {
        self.brightness -= 1;
    }

    fn get_brightness(&self) -> i32 {
        return self.brightness;
    }

    fn turn_off(&mut self) {
        self.brightness = 0;
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

    fn turn_off(&mut self) {
        self.r = 0;
        self.g = 0;
        self.b = 0;
    }

    fn get_brightness(&self) -> i32 {
        return (self.r + self.g + self.b) / 3;
    }
}

pub fn demo() {
    let mut light = Dimmer { brightness: 0 };
    light.increase_brightness();

    println!("Brightness: {}", light.get_brightness());
}
