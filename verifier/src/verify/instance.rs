use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;
use rug;
use serde::{Deserialize, Serialize};
use serde_with;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "verify/gh_ll_instance.pest"]
pub struct InstanceParser;

#[serde_with::serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

pub const PRECISION: u32 = 128;

pub fn fl(val: i32) -> rug::Float {
    rug::Float::with_val(PRECISION, val)
}

impl Point {
    pub fn dist(&self, other: &Self) -> rug::Float {
        let xs = self.x - other.x;
        let ys = self.y - other.y;
        fl(xs * xs + ys * ys).sqrt()
    }
}

pub fn calc_route_distance(inst: &Instance, route: &Vec<usize>) -> rug::Float {
    let depot = &inst.pts[0];
    let first = &inst.pts[route[0]];

    let l = *route.last().unwrap();
    let last = &inst.pts[l];

    let route_distance = route
        .iter()
        .map(|&p| &inst.pts[p])
        .tuple_windows()
        .map(|(from, to)| from.dist(to))
        .reduce(std::ops::Add::add)
        .unwrap_or(fl(0));

    depot.dist(first) + route_distance + last.dist(depot)
}

pub fn check_route_time(
    inst: &Instance,
    route_id: usize,
    route: &Vec<usize>,
) -> Result<(), String> {
    let depot = &inst.pts[0];
    let first = &inst.pts[route[0]];
    let mut time = fl(depot.start + depot.service);
    time += depot.dist(first);

    if time > first.due as f64 {
        Err(format!(
            "arrived too late ({}) at {} in route {} at position 0",
            time, first.id, route_id,
        ))?;
    }

    time = time.max(&rug::Float::with_val(PRECISION, first.start));

    time += first.service;

    for ((_, f), (tidx, t)) in route.iter().enumerate().tuple_windows() {
        let from = &inst.pts[*f];
        let to = &inst.pts[*t];

        time += from.dist(to);

        if time > to.due as f64 {
            Err(format!(
                "arrived too late ({}) at {} in route {} at position {}",
                time, to.id, route_id, tidx
            ))?;
        }

        time = time.max(&fl(to.start));
        time += to.service;
    }

    let l = *route.last().unwrap();
    let last = &inst.pts[l];
    time += last.dist(&depot);
    if time > depot.due as f64 {
        Err(format!(
            "arrived too late ({}) in route {} at depot",
            time, route_id,
        ))?;
    }

    Ok(())
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
                Err(_) => return Err(format!("can't parse line `{s}': error in trying to parse field {i}: `{c}' can not be parsed ")),
            };
        }

        let nums = vs.len();

        if nums != 7 && nums != 9 {
            Err(format!(
                "expected 7 or 9 integers in line `{s}', have {nums} numbers"
            ))?;
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
        let parsed = InstanceParser::parse(Rule::file, &s)
            .map_err(|x| format!("Instance parsing problem: {x}"))?
            .next()
            .unwrap();

        let mut pts: Vec<Point> = vec![];
        let mut v: Vec<i32> = vec![];
        let mut name = "".to_string();

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
                Rule::instance_name => {
                    name = r.as_span().as_str().to_string();
                }
                Rule::d => {}
                _ => unreachable!(),
            }
        }
        let inst = Instance {
            name,
            vehicles: v[0],
            max_capacity: v[1],
            is_pdp: pts[0].pickup_delivery.is_some(),
            pts,
        };
        inst.check_sanity()?;
        Ok(inst)
    }
}

impl Instance {
    fn point_ids_are_sequential(&self) -> Result<(), String> {
        let pts: Vec<usize> = self
            .pts
            .iter()
            .enumerate()
            .filter_map(|(i, pt)| if pt.id as usize != i { Some(i) } else { None })
            .collect();

        if pts.is_empty() {
            Ok(())
        } else {
            Err(format!("points {:?} do not have correct ids", pts))
        }
    }

    fn check_demands(&self) -> Result<(), String> {
        for pt in self.pts.iter() {
            if pt.demand > self.max_capacity {
                Err(format!("point {} can not be visited because its demands are greater than vehicle capacity", pt.id))?;
            }

            if pt.demand < 0 && !self.is_pdp {
                Err(format!(
                    "point {} has negative demands and this is not pdp",
                    pt.id
                ))?;
            }

            if !self.is_pdp && pt.pickup_delivery.is_some() {
                Err(format!(
                    "point {} has pickup and delivery but the instance is not pdp",
                    pt.id
                ))?;
            }

            if let Some((p, d)) = pt.pickup_delivery {
                if p != 0 && d != 0 {
                    Err(format!(
                        "point {} has nonzero both pickup ({p}) and delivery ({d})",
                        pt.id
                    ))?;
                }

                let other_idx = if p != 0 { p } else { d };

                if other_idx < 0 || other_idx >= self.pts.len() as i32 {
                    Err(format!(
                        "points {} pdp pair {other_idx} does not refer to any legal point",
                        pt.id,
                    ))?;
                }

                let other = &self.pts[other_idx as usize];

                if other.pickup_delivery.is_none() {
                    Err(format!("point {} pdp pair {other_idx} is not pdp", pt.id,))?;
                }

                if (other.pickup_delivery != Some((0, pt.id)))
                    && (other.pickup_delivery != Some((pt.id, 0)))
                {
                    Err(format!("point {} and {other_idx} are a pdp pair but their pickup and deliveries do not match", pt.id,))?;
                }

                let others_demand = other.demand;
                if pt.demand + others_demand != 0 {
                    Err(format!(
                        "point {} demands {} does not sum to 0 with ther pdp pair {other} demands {others_demand}",
                        pt.id,
                        pt.demand,
                    ))?;
                }
            }
        }

        if self.is_pdp && self.pts[0].pickup_delivery != Some((0, 0)) {
            Err(format!("depots pdp pair is not (0, 0)"))?;
        }

        let depots_demand = self.pts[0].demand;

        if depots_demand != 0 {
            Err(format!("depots demand is non-zero ({depots_demand})"))?;
        }

        Ok(())
    }

    fn check_time(&self) -> Result<(), String> {
        for pt in self.pts.iter() {
            if pt.start > pt.due {
                Err(format!(
                    "point {} can not be visited because the due time ({}) is before start ({})",
                    pt.id, pt.due, pt.start
                ))?;
            }

            let depot = &self.pts[0];

            let earliest_arrival = depot.start + depot.dist(pt);
            if earliest_arrival > pt.due {
                Err(format!(
                    "earliest possible arrival ({earliest_arrival}) from depot to point {} is after the points due time {}",
                    pt.id, pt.due
                ))?;
            }

            let earliest_service_finish = fl(pt.start).max(&earliest_arrival) + pt.service;
            let earliest_return = earliest_service_finish + pt.dist(&depot);

            if earliest_return > depot.due {
                Err(format!(
                    "earliest possible return to depot ({earliest_return}) to point {} is after the depot due time {}",
                    pt.id, depot.due
                ))?;
            }
        }
        Ok(())
    }

    pub fn check_sanity(&self) -> Result<(), String> {
        let clients = self.pts.len();
        if clients < 2 {
            Err(format!(
                "the instance needs at least two points (depot and one client to visit), it has {}",
                clients
            ))?;
        }
        self.point_ids_are_sequential()?;
        self.check_demands()?;
        self.check_time()?;
        Ok(())
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
            Err(format!(
                "expected 7 or 9 integers in line `{line}', have 4 numbers"
            ))
        );
    }

    #[test]
    fn read_gh_point_8_values() {
        let line = "0 1 2 3 4 5 6 7";
        let point = Point::from_str(line);

        assert_eq!(
            point,
            Err(format!(
                "expected 7 or 9 integers in line `{line}', have 8 numbers"
            ))
        );
    }

    #[test]
    fn read_gh_point_too_many() {
        let line = "0 1 2 3 4 5 6 7 8 9";
        let point = Point::from_str(line);

        assert_eq!(
            point,
            Err(format!(
                "expected 7 or 9 integers in line `{line}', have 10 numbers"
            ))
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

    #[test]
    fn read_gh_instance() {
        let instance = concat!(
            "c1_1_1\n",
            "\n",
            "VEHICLE\n",
            "NUMBER CAPACITY\n",
            "12 100\n",
            "\n",
            "CUSTOMER\n",
            "CUST NO.  XCOORD.    YCOORD.    DEMAND   READY TIME  DUE DATE   SERVICE TIME\n",
            "\n",
            "0 1 2 0 4 100 6\n",
            "1 2 3 4 5 6 7\n",
            "2 3 4 5 6 7 8\n",
            "3 4 5 6 7 10 9\n"
        );
        let inst = Instance::from_str(instance);

        assert_eq!(inst.as_ref().err(), None);

        assert_eq!(
            inst.unwrap(),
            Instance {
                name: String::from(""),
                vehicles: 12,
                max_capacity: 100,
                pts: vec![
                    Point {
                        id: 0,
                        x: 1,
                        y: 2,
                        demand: 0,
                        start: 4,
                        due: 100,
                        service: 6,
                        pickup_delivery: None,
                    },
                    Point {
                        id: 1,
                        x: 2,
                        y: 3,
                        demand: 4,
                        start: 5,
                        due: 6,
                        service: 7,
                        pickup_delivery: None,
                    },
                    Point {
                        id: 2,
                        x: 3,
                        y: 4,
                        demand: 5,
                        start: 6,
                        due: 7,
                        service: 8,
                        pickup_delivery: None,
                    },
                    Point {
                        id: 3,
                        x: 4,
                        y: 5,
                        demand: 6,
                        start: 7,
                        due: 10,
                        service: 9,
                        pickup_delivery: None,
                    },
                ],
                is_pdp: false,
            }
        );
    }

    #[test]
    fn read_ll_instance() {
        let instance = concat!(
            "12\t100\n",
            "0\t1\t2\t0\t4\t100\t6\t0\t0\n",
            "1\t2\t3\t4\t5\t6\t7\t0\t2\n",
            "2\t3\t4\t-4\t6\t7\t8\t1\t0\n",
            "3\t4\t5\t6\t7\t10\t9\t0\t4\n",
            "4\t5\t6\t-6\t8\t10\t10\t3\t0\n",
        );
        let inst = Instance::from_str(instance);

        assert_eq!(inst.as_ref().err(), None);

        assert_eq!(
            inst.unwrap(),
            Instance {
                name: String::from(""),
                vehicles: 12,
                max_capacity: 100,
                pts: vec![
                    Point {
                        id: 0,
                        x: 1,
                        y: 2,
                        demand: 0,
                        start: 4,
                        due: 100,
                        service: 6,
                        pickup_delivery: Some((0, 0)),
                    },
                    Point {
                        id: 1,
                        x: 2,
                        y: 3,
                        demand: 4,
                        start: 5,
                        due: 6,
                        service: 7,
                        pickup_delivery: Some((0, 2)),
                    },
                    Point {
                        id: 2,
                        x: 3,
                        y: 4,
                        demand: -4,
                        start: 6,
                        due: 7,
                        service: 8,
                        pickup_delivery: Some((1, 0)),
                    },
                    Point {
                        id: 3,
                        x: 4,
                        y: 5,
                        demand: 6,
                        start: 7,
                        due: 10,
                        service: 9,
                        pickup_delivery: Some((0, 4)),
                    },
                    Point {
                        id: 4,
                        x: 5,
                        y: 6,
                        demand: -6,
                        start: 8,
                        due: 10,
                        service: 10,
                        pickup_delivery: Some((3, 0)),
                    },
                ],
                is_pdp: true,
            }
        );
    }
}
