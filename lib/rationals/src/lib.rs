#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ConstRational {
    numerator: i32,
    denominator: i32,
}

impl ConstRational {
    pub const fn new(numerator: i32, denominator: i32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
    pub const fn add(self, rhs: Self) -> Self {
        Self::new(
            self.numerator * rhs.denominator + rhs.numerator * self.denominator,
            self.denominator * rhs.denominator,
        )
    }
    pub const fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.numerator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }
    pub const fn neg(self) -> Self {
        Self::new(-self.numerator, self.denominator)
    }

    pub const fn sub(self, rhs: Self) -> Self {
        self.add(rhs.neg())
    }

    pub const fn invert(self) -> Self {
        Self::new(self.denominator, self.numerator)
    }

    pub const fn div(self, rhs: Self) -> Self {
        self.mul(rhs.invert())
    }
    pub fn into_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
    pub const fn normalize(self) -> Self {
        let gcd = gcd(self.numerator, self.denominator);
        Self::new(self.numerator / gcd, self.denominator / gcd)
    }
    pub const fn floor(self) -> i32 {
        self.numerator / self.denominator
    }
    pub const fn ge(self, other: Self) -> bool {
        let a = self.numerator * other.denominator;
        let b = other.numerator * self.denominator;
        a >= b
    }
    pub const fn le(self, other: Self) -> bool {
        let a = self.numerator * other.denominator;
        let b = other.numerator * self.denominator;
        a <= b
    }
}

const fn gcd(a: i32, b: i32) -> i32 {
    match (a, b) {
        (0, x) => x,
        (x, y) => gcd((x - y).abs(), min(x, y)),
    }
}

const fn min(a: i32, b: i32) -> i32 {
    if a > b {
        b
    } else {
        a
    }
}
