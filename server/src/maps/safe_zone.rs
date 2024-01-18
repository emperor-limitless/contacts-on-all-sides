pub struct SafeZone {
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
}
impl SafeZone {
    pub fn new(min_x: isize, max_x: isize, min_y: isize, max_y: isize) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
    pub fn in_range(&self, x: isize, y: isize) -> bool {
        if x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y {
            return true;
        }
        false
    }
}
