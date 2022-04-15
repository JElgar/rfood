trait Light {
    fn get_brightness(&self) -> i32;
    fn set_brightness(&mut self, brightness: i32);
}

struct Dimmer {
    brightness: i32,
}

impl Light for Dimmer {
    fn set_brightness(&mut self, brightness: i32) {
        self.brightness = brightness;
    }
    fn get_brightness(&self) -> i32 {
        return self.brightness;
    }
}
