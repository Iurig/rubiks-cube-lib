pub(crate) trait RubiksCubeCleaning {
    fn process_movement_input(&self) -> impl Iterator<Item = String>;
}

fn expand_wide_moves(m: &str) -> String {
    let mut replaced = String::new();
    for c in m.chars() {
        match c {
            'u' | 'f' | 'r' | 'b' | 'l' => {
                replaced.push(c.to_ascii_uppercase());
                replaced.push('w');
            }
            _ => replaced.push(c),
        }
    }
    replaced
}

impl<T: AsRef<str> + ?Sized> RubiksCubeCleaning for T {
    fn process_movement_input(&self) -> impl Iterator<Item = String> {
        self.as_ref()
            .lines()
            .flat_map(|l| l.split("//").next().unwrap_or("").split_whitespace())
            .map(expand_wide_moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_line_comment_removal() {
        let with_comment: Vec<String> =
            "R U //this is a comment".process_movement_input().collect();
        let without_comment: Vec<String> = "R U".process_movement_input().collect();
        assert_eq!(
            with_comment, without_comment,
            "strings were ' {with_comment:?} '  and ' {without_comment:?} '"
        );
    }

    #[test]
    fn multiline_comment_removal() {
        let with_comment: Vec<String> = "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP"
            .process_movement_input()
            .collect();
        let without_comment: Vec<String> = "y2 F' M F' R U' R U' Fw z'
            U R U r M' U' R U2' R'
            U R' U' R U' R' U' r
            U M' U' M U' U' M' U M
            U' U' M2' U' M U' U' M' U' U' M2'"
            .process_movement_input()
            .collect();
        assert_eq!(
            with_comment, without_comment,
            "strings were ' {with_comment:?} '  and ' {without_comment:?} '"
        );
    }
}
