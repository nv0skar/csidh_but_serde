use num_bigint::BigUint;
use num_traits::identities::{One, Zero};

use crate::global;
use crate::ffield::{Field, FieldElement};

#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    field: Field,
    a: BigUint,
    b: BigUint,
}

impl Curve {
    pub fn new(field: Field, a: BigUint, b: BigUint) -> Curve {
        Curve {
            field, a, b,
        }
    }

    pub fn contains(&self, p: &Point) -> bool {
        let left = (&p.y * &p.y) * &self.b;
        let right = &p.x * &p.x * &p.x + &p.x * &p.x * &self.a + &p.x;

        left == right
    }

    fn recover(p: &Point, q: &ProjectivePoint, o: &ProjectivePoint) -> Point {
        let v1 = &p.x * &q.z;
        let v2 = &q.x + &v1;
        let v3 = &q.x - &v1;
        let v3 = &v3 * &v3;
        let v3 = &v3 * &o.x;
        let v1 = &q.z * BigUint::from(2u32) * &p.curve.a;
        let v2 = &v2 + &v1;
        let v4 = &p.x * &q.x;
        let v4 = &v4 + &q.z;
        let v2 = &v2 * &v4;
        let v1 = &v1 * &q.z;
        let v2 = &v2 - &v1;
        let v2 = &v2 * &o.z;
        let y  = &v2 - &v3;
        let v1 = &p.y * BigUint::from(2u32) * &p.curve.b;
        let v1 = &v1 * &q.z;
        let v1 = &v1 * &o.z;
        let x  = &v1 * &q.x;
        let z  = &v1 * &q.z;

        Point {
            curve: p.curve.clone(),
            x,
            y,
            z,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    curve: Curve,
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
}

impl Point {
    pub fn new(curve: Curve, x: BigUint, y: BigUint) -> Point {
        Point {
            x: curve.field.get(x),
            y: curve.field.get(y),
            z: curve.field.get(1u32),
            curve,
        }
    }

    pub fn multiply(&self, k: &BigUint) -> Point {
        let (x0, x1) = Point::ladder(&k, self.projectivize());
        let q = Curve::recover(self, &x0, &x1);
        return q;
    }

    fn ladder(k: &BigUint, p: ProjectivePoint) -> (ProjectivePoint, ProjectivePoint) {
        let mut x0 = p.clone();
        let mut x1 = p.double();
        let l = k.bits();
        assert!(k >> (l-1) == BigUint::one());

        // BUG! Rust can't do backwards ranges...
        for i in (0..=(l-2)).rev() {
            let b = (k >> i) & BigUint::one();
            if b == BigUint::zero() {
                let temp = x0.add(&x1, &p);
                x1 = temp;
                x0 = x0.double();
            } else {
                x0 = x0.add(&x1, &p);
                x1 = x1.double();
            }
        }

        return (x0, x1);
    }

    fn projectivize(&self) -> ProjectivePoint {

        let zero = self.curve.field.get(0u32);

        if self.x == zero || self.z == zero {
            ProjectivePoint {
                curve: self.curve.clone(),
                x: self.curve.field.get(1u32),
                z: self.curve.field.get(0u32),
            }
        } else {
            ProjectivePoint {
                curve: self.curve.clone(),
                x: self.x.clone(),
                z: self.curve.field.get(1u32),
            }
        }
    }

    fn unproject(self) -> Point {
        assert!(self.z != self.curve.field.get(0u32));

        println!("{} {} {}", self.x, self.y, self.z);

        let x = &self.x / &self.z;
        let y = &self.y / &self.z;
        let z = &self.z / &self.z;

        println!("{} {} {}", x, y, z);
        Point {
            x, y, z,
            curve: self.curve,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ProjectivePoint {
    curve: Curve,
    x: FieldElement,
    z: FieldElement,
}

impl ProjectivePoint {

    pub fn add(&self, other: &ProjectivePoint, orig: &ProjectivePoint) -> ProjectivePoint {
        let v0 = &self.x + &self.z;
        let v1 = &other.x - &other.z;
        let v1 = &v1 * &v0;
        let v0 = &self.x - &self.z;
        let v2 = &other.x + &other.z;
        let v2 = &v2 * &v0;
        let v3 = &v1 + &v2;
        let v3 = &v3 * &v3;
        let v4 = &v1 - &v2;
        let v4 = &v4 * &v4;
        let x = &orig.z * &v3;
        let z = &orig.x * &v4;

        assert!(x != self.curve.field.get(0u32));
        assert!(z != self.curve.field.get(0u32));
        return ProjectivePoint {
            curve: self.curve.clone(),
            x,
            z,
        }
    }

    pub fn double(&self) -> ProjectivePoint {
        let v1 = &self.x + &self.z;
        let v1 = &v1 * &v1;
        let v2 = &self.x - &self.z;
        let v2 = &v2 * &v2;
        let x = &v1 * &v2;
        let v1 = &v1 - &v2;
        // Here was a bug! I forgot that division -> Modulo Ring
        let a_2 = self.curve.field.get(&self.curve.a + 2u32);
        let v3 = &v1 * &(&a_2 / &self.curve.field.get(4u32));
        let v3 = &v3 + &v2;
        let z = &v1 * &v3;

        assert!(x != self.curve.field.get(0u32));
        assert!(z != self.curve.field.get(0u32));
        return ProjectivePoint {
            curve: self.curve.clone(),
            x,
            z,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_mul() {
        let x = BigUint::parse_bytes(b"2051044887188588280366899510711463515184102432059522841387541984999186019238289110841661333718393379209806643406155944602233875537370058705956384966209858", 10).unwrap();
        let y = BigUint::parse_bytes(b"2999054700883294606115636709285947688603015463995111523694534197644452886751843273757676343103953201273958036952062931228773567734286840492294219977378136", 10).unwrap();

        let other_x = BigUint::parse_bytes(b"1254817631949275079030490581963578364746575569014839158947538007979236709253796922466332191140273712204313677321924940880514829958528954596325165920058277", 10).unwrap();
        let other_y = BigUint::parse_bytes(b"2381495309685763751265865484184529659090354786855457591442552214156841700513768692570497752099605704710183797526595611214891101033449784504091079214700929", 10).unwrap();

        let field = Field::new(global::p.clone());
        let a = 0u32.into();
        let b = 1u32.into();
        let curve = Curve::new(field, a, b);
        let point = Point::new(curve.clone(), x, y);
        assert!(curve.contains(&point));
        let other_point = Point::new(curve.clone(), other_x, other_y);
        assert!(curve.contains(&other_point));

        let multiplied = point.multiply(&BigUint::from(9u32)).unproject();

        println!("X: {}\nY: {}\nZ: {}\n\nX: {}\nY: {}\nZ: {}",
                 multiplied.x, multiplied.y, multiplied.z,
                 other_point.x, other_point.y, other_point.z);

        assert_eq!(other_point, multiplied);
    }
}
