use pest::Parser;
use pest_derive::Parser;
use rug;
use serde::{Deserialize, Serialize};
use serde_with;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "gh_ll_instance.pest"]
pub struct InstanceParser;

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Point {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub demand: i32,
    pub start: i32,
    pub due: i32,
    pub service: i32,
    pub pickup_delivery: Option<(i32, i32)>,
}

pub const PRECISION: u32 = 100;

impl Point {
    pub fn dist(&self, other: &Self) -> rug::Float {
        let xs = self.x - other.x;
        let ys = self.y - other.y;
        rug::Float::with_val(PRECISION, xs * xs + ys * ys).sqrt()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.pickup_delivery {
            None => write!(
                f,
                "{:5} {:7} {:10} {:10} {:10} {:10} {:10}\n",
                self.id, self.x, self.y, self.demand, self.start, self.due, self.service
            ),
            Some((pickup, delivery)) => write!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                self.id,
                self.x,
                self.y,
                self.demand,
                self.start,
                self.due,
                self.service,
                pickup,
                delivery
            ),
        }
    }
}

impl FromStr for Point {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut vs: Vec<i32> = Vec::with_capacity(9);

        for (i, c) in s.split_whitespace().into_iter().enumerate() {
            match c.parse() {
                Ok(n) => vs.push(n),
                Err(_) => return Err(format!("can't parse line `{s}': error in trying to parse field {i}: `{c}' can not be parsed ").to_string()),
            };
        }

        let nums = vs.len();

        if nums != 7 && nums != 9 {
            return Err(
                format!("expected 7 or 9 integers in line `{s}', have {nums} numbers")
                    .to_string(),
            );
        }

        Ok(Point {
            id: vs[0],
            x: vs[1],
            y: vs[2],
            demand: vs[3],
            start: vs[4],
            due: vs[5],
            service: vs[6],
            pickup_delivery: if vs.len() > 7 {
                Some((vs[7], vs[8]))
            } else {
                None
            },
        })
    }
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Instance {
    pub name: String,
    pub vehicles: i32,
    pub max_capacity: i32,
    pub pts: Vec<Point>,
    pub is_pdp: bool,
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_pdp {
            write! {f, "{}\t{}\t0\n", &self.vehicles, self.max_capacity}?;
        } else {
            write! {f, "{}\n\nVEHICLE\nNUMBER     CAPACITY\n{:4}{:13}\n\nCUSTOMER\nCUST NO.  XCOORD.    YCOORD.    DEMAND   READY TIME  DUE DATE   SERVICE TIME\n\n", &self.name, self.vehicles, self.max_capacity}?;
        };
        for pt in self.pts.iter() {
            write!(f, "{}", pt)?;
        }
        Ok(())
    }
}

impl FromStr for Instance {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pr = InstanceParser::parse(Rule::file, &s);

        let parsed = match pr {
            Ok(x) => x,
            Err(x) => {
                return Err(x.to_string());
            }
        }
        .next()
        .unwrap();

        let mut pts: Vec<Point> = vec![];
        let mut v: Vec<i32> = vec![];

        for r in parsed.into_inner() {
            match r.as_rule() {
                Rule::vehicles_capacity => {
                    v = r
                        .as_span()
                        .as_str()
                        .split_whitespace()
                        .into_iter()
                        .map(|c| c.parse().unwrap_or_default())
                        .collect();
                }
                Rule::row => {
                    pts.push(r.as_span().as_str().parse().unwrap());
                }
                Rule::d => {}
                _ => unreachable!(),
            }
        }

        Ok(Instance {
            name: "".to_string(),
            vehicles: v[0],
            max_capacity: v[1],
            is_pdp: pts[0].pickup_delivery.is_some(),
            pts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_gh_point() {
        let line = " 0    1      2    3   4   5  6";
        let point = Point::from_str(line).unwrap();

        assert_eq!(
            point,
            Point {
                id: 0,
                x: 1,
                y: 2,
                demand: 3,
                start: 4,
                due: 5,
                service: 6,
                pickup_delivery: None
            }
        );
    }

    #[test]
    fn read_gh_point_not_enough() {
        let line = "0 1 2 3";
        let point = Point::from_str(line);

        assert_eq!(
            point,
            Err(
                format!("expected 7 or 9 integers in line `{line}', have 4 numbers")
            )
        );
    }

    #[test]
    fn read_gh_point_8_values() {
        let line = "0 1 2 3 4 5 6 7";
        let point = Point::from_str(line);

        assert_eq!(
            point,
            Err(
                format!("expected 7 or 9 integers in line `{line}', have 8 numbers")
            )
        );
    }


    #[test]
    fn read_gh_point_too_many() {
        let line = "0 1 2 3 4 5 6 7 8 9";
        let point = Point::from_str(line);

        assert_eq!(
            point,
            Err(
                format!("expected 7 or 9 integers in line `{line}', have 10 numbers")
            )
        );
    }

    #[test]
    fn read_ll_point() {
        let line = "0\t1\t2\t3\t4\t5\t6\t7\t8";
        let point = Point::from_str(line).unwrap();

        assert_eq!(
            point,
            Point {
                id: 0,
                x: 1,
                y: 2,
                demand: 3,
                start: 4,
                due: 5,
                service: 6,
                pickup_delivery: Some((7, 8))
            }
        );
    }
}
