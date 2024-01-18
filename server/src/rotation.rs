pub fn get_1d_distance(x1: isize, x2: isize) -> isize {
    (x1 - x2).abs()
}

pub fn get_2d_distance(x1: isize, y1: isize, x2: isize, y2: isize) -> isize {
    let x = get_1d_distance(x1, x2);
    let y = get_1d_distance(y1, y2);
    x + y
}
