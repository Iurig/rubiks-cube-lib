pub fn ordinal(number: usize) -> String {
    let ending = match number % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{number}{ending}")
}
