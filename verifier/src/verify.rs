pub mod instance;
pub mod solution;
use instance::{fl, Instance};
use itertools::Itertools;
use solution::Solution;

pub fn calc_route_distance(inst: &Instance, route: &Vec<usize>) -> rug::Float {
    let depot = &inst.pts[0];
    let first = &inst.pts[route[0]];

    let last_idx = *route.last().unwrap();
    let last = &inst.pts[last_idx];

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

    time = time.max(&fl(first.start));

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

fn check_route_load(inst: &Instance, route_id: usize, route: &Vec<usize>) -> Result<(), String> {
    let mut vehicle_load = 0;
    for (p, pt) in route.iter().map(|&p_id| &inst.pts[p_id]).enumerate() {
        vehicle_load += pt.demand;
        if vehicle_load < 0 {
            Err(format!(
                "current load is negative at {} in route {} at position {}",
                pt.id, route_id, p,
            ))?;
        }

        if vehicle_load > inst.max_capacity {
            Err(format!(
                "load is greater than max load ({} > {}) at {} in route {} at position {}",
                vehicle_load, inst.max_capacity, pt.id, route_id, p,
            ))?;
        }
    }
    Ok(())
}

fn check_pdp(inst: &Instance, sol: &Solution) -> Result<(), String> {
    let mut point_route_id = vec![0; inst.pts.len()];
    let mut route_idx = vec![0; inst.pts.len()];

    for (route_id, route) in sol.routes.iter().enumerate() {
        for (i, &p) in route.iter().enumerate() {
            point_route_id[p] = route_id + 1;
            route_idx[p] = i;
        }
    }

    for pt in 1..point_route_id.len() {
        let (p, d) = inst.pts[pt].pickup_delivery.unwrap();

        let (pickup, delivery) = if p != 0 {
            (p as usize, pt)
        } else {
            (pt, d as usize)
        };

        if point_route_id[pickup] != point_route_id[delivery] {
            Err(format!(
                "pickup {} and delivery {} are not in the same routes (are in routes {} and {})",
                pickup, delivery, point_route_id[pickup], point_route_id[delivery],
            ))?
        }

        if route_idx[pickup] > route_idx[delivery] {
            Err(format!(
                "delivery {} is before its pickup {} (are on positions {} and {})",
                delivery, pickup, route_idx[delivery], route_idx[pickup],
            ))?
        }
    }

    Ok(())
}

fn check_basic_sanity(inst: &Instance, sol: &Solution) -> Result<(), String> {
    let mut point_route_id = vec![None; inst.pts.len()];

    point_route_id[0] = Some(0);

    for (route_id, route) in sol.routes.iter().enumerate() {
        for (r, &pt) in route.iter().enumerate() {
            if pt == 0 {
                Err(format!(
                    "route {} visits depot at non-terminal position {}",
                    route_id + 1,
                    r
                ))?;
            }

            if pt > point_route_id.len() {
                Err(format!(
                    "node {} in route {} at position {} is not described in the instance",
                    pt,
                    route_id + 1,
                    r
                ))?;
            }

            match point_route_id[pt] {
                None => point_route_id[pt] = Some(route_id + 1),
                Some(other_route) => Err(format!(
                    "node {} visited at least two times (in routes {} and {})",
                    pt,
                    route_id + 1,
                    other_route
                ))?,
            }
        }
    }

    for (pt, visited) in point_route_id.iter().enumerate() {
        if visited.is_none() {
            Err(format!("node {} not visited in any route", pt,))?;
        }
    }

    Ok(())
}

pub fn verify(inst: &Instance, sol: &Solution) -> Result<rug::Float, String> {
    check_basic_sanity(&inst, &sol)?;

    if inst.is_pdp {
        check_pdp(&inst, &sol)?;
    }

    if sol.routes.len() > inst.vehicles as usize {
        Err(format!(
            "more vehicles than allowed ({} > {})",
            sol.routes.len(),
            inst.vehicles
        ))?;
    }

    let mut total_distance = fl(0);
    for (route_id, route) in sol.routes.iter().enumerate() {
        check_route_time(&inst, route_id + 1, &route)?;
        check_route_load(&inst, route_id + 1, &route)?;

        total_distance += calc_route_distance(inst, &route);
    }

    Ok(total_distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use instance::Point;

    fn setup() -> Instance {
        let inst = Instance {
            name: "test".to_string(),
            is_pdp: false,
            vehicles: 3,
            max_capacity: 10,
            pts: vec![
                Point {
                    id: 0,
                    x: 0,
                    y: 0,
                    demand: 0,
                    start: 0,
                    due: 48,
                    service: 0,
                    pickup_delivery: None,
                },
                Point {
                    id: 1,
                    x: 0,
                    y: 1,
                    demand: 2,
                    start: 0,
                    due: 10,
                    service: 10,
                    pickup_delivery: None,
                },
                Point {
                    id: 2,
                    x: 1,
                    y: 1,
                    demand: 2,
                    start: 0,
                    due: 3600,
                    service: 10,
                    pickup_delivery: None,
                },
                Point {
                    id: 3,
                    x: 1,
                    y: 0,
                    demand: 2,
                    start: 0,
                    due: 3600,
                    service: 10,
                    pickup_delivery: None,
                },
                Point {
                    id: 4,
                    x: 0,
                    y: -1,
                    demand: 2,
                    start: 0,
                    due: 3600,
                    service: 10,
                    pickup_delivery: None,
                },
                Point {
                    id: 5,
                    x: -1,
                    y: -1,
                    demand: 2,
                    start: 0,
                    due: 3600,
                    service: 10,
                    pickup_delivery: None,
                },
                Point {
                    id: 6,
                    x: -1,
                    y: 0,
                    demand: 2,
                    start: 0,
                    due: 3600,
                    service: 10,
                    pickup_delivery: None,
                },
            ],
        };
        assert_eq!(inst.check_sanity(), Ok(()));
        inst
    }

    #[test]
    fn verify_correct() {
        let inst = setup();

        let res = verify(
            &inst,
            &Solution {
                routes: vec![vec![1, 2, 3], vec![4, 5, 6]],
                ..Default::default()
            },
        );

        assert_eq!(res, Ok(fl(8)));
    }

    #[test]
    fn test_check_basic_sanity_errors() {
        let inst = setup();

        assert_eq!(
            check_basic_sanity(
                &inst,
                &Solution {
                    routes: vec![vec![1, 2, 0, 3], vec![4, 5, 6]],
                    ..Default::default()
                },
            ),
            Err("route 1 visits depot at non-terminal position 2".to_string())
        );

        assert_eq!(
            check_basic_sanity(
                &inst,
                &Solution {
                    routes: vec![vec![1, 2, 3], vec![4, 5, 60]],
                    ..Default::default()
                },
            ),
            Err("node 60 in route 2 at position 2 is not described in the instance".to_string())
        );

        assert_eq!(
            check_basic_sanity(
                &inst,
                &Solution {
                    routes: vec![vec![1, 2, 3], vec![4, 5, 3, 6]],
                    ..Default::default()
                },
            ),
            Err("node 3 visited at least two times (in routes 2 and 1)".to_string())
        );
        assert_eq!(
            check_basic_sanity(
                &inst,
                &Solution {
                    routes: vec![vec![1, 2, 3, 1], vec![4, 5, 6]],
                    ..Default::default()
                },
            ),
            Err("node 1 visited at least two times (in routes 1 and 1)".to_string())
        );

        assert_eq!(
            check_basic_sanity(
                &inst,
                &Solution {
                    routes: vec![vec![1, 2, 3], vec![4, 6]],
                    ..Default::default()
                },
            ),
            Err("node 5 not visited in any route".to_string())
        );
    }

    #[test]
    fn too_many_vehicles() {
        let inst = setup();

        let res = verify(
            &inst,
            &Solution {
                routes: (1..=6).map(|x| vec![x]).collect(),
                ..Default::default()
            },
        );

        assert_eq!(res, Err("more vehicles than allowed (6 > 3)".to_string()));
    }

    #[test]
    fn routes_too_large_load() {
        let inst = setup();

        let res = check_route_load(&inst, 1, &(1..=6).collect());

        assert_eq!(
            res,
            Err(
                "load is greater than max load (12 > 10) at 6 in route 1 at position 5".to_string()
            )
        );
    }

    #[test]
    fn routes_time() {
        let inst = setup();

        let res = check_route_time(&inst, 1, &vec![1, 2, 3, 6, 5, 4]);

        assert_eq!(
            res,
            Err(
                "arrived too late (68.00000000000000000000000000000000000000) in route 1 at depot"
                    .to_string()
            )
        );

        let res = check_route_time(&inst, 2, &vec![3, 2, 1]);

        assert_eq!(res, Err("arrived too late (23.00000000000000000000000000000000000000) at 1 in route 2 at position 2".to_string()));
    }

    #[test]
    fn pdp() {
        let mut inst = setup();
        inst.is_pdp = true;
        inst.pts[0].pickup_delivery = Some((0, 0));
        inst.pts[1].pickup_delivery = Some((0, 2));
        inst.pts[2].pickup_delivery = Some((1, 0));
        inst.pts[2].demand = -2;
        inst.pts[3].pickup_delivery = Some((0, 4));
        inst.pts[4].pickup_delivery = Some((3, 0));
        inst.pts[4].demand = -2;
        inst.pts[5].pickup_delivery = Some((0, 6));
        inst.pts[6].pickup_delivery = Some((5, 0));
        inst.pts[6].demand = -2;

        let res = check_pdp(
            &inst,
            &Solution {
                routes: vec![vec![1, 2, 3], vec![4, 5, 6]],
                ..Default::default()
            },
        );

        assert_eq!(
            res,
            Err(
                "pickup 3 and delivery 4 are not in the same routes (are in routes 1 and 2)"
                    .to_string()
            )
        );

        let res = check_pdp(
            &inst,
            &Solution {
                routes: vec![vec![1, 2, 3, 4], vec![6, 5]],
                ..Default::default()
            },
        );

        assert_eq!(
            res,
            Err("delivery 6 is before its pickup 5 (are on positions 0 and 1)".to_string())
        );

        let res = check_route_load(&inst, 1, &vec![3, 2, 6, 5, 4, 1]);

        assert_eq!(
            res,
            Err(("current load is negative at 6 in route 1 at position 2").to_string())
        );

        let res = check_route_load(&inst, 1, &vec![3, 6, 5, 4]);

        assert_eq!(res, Ok(()));
    }
}
