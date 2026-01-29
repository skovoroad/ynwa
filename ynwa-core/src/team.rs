#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Team {
    A,
    B,
}

impl Team {
    pub fn opposite(&self) -> Team {
        match self {
            Team::A => Team::B,
            Team::B => Team::A,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposite() {
        assert_eq!(Team::A.opposite(), Team::B);
        assert_eq!(Team::B.opposite(), Team::A);
    }
}
