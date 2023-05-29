pub(crate) const fn div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::div_ceil;

    #[test]
    fn div_ceil_works() {
        assert_eq!(div_ceil(2, 2), 1);
        assert_eq!(div_ceil(2, 4), 1);
        assert_eq!(div_ceil(3, 2), 2);
        assert_eq!(div_ceil(4, 2), 2);
        assert_eq!(div_ceil(17, 8), 3);
    }
}
