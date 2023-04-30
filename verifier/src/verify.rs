pub mod instance;
pub mod solution;
use instance::{fl, Instance};
use solution::Solution;
use itertools::Itertools;

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
            point_route_id[p] = route_id;
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

    let mut total_distance = rug::Float::new(instance::PRECISION);
    for (route_id, route) in sol.routes.iter().enumerate() {
        check_route_time(&inst, route_id + 1, &route)?;
        check_route_load(&inst, route_id + 1, &route)?;

        total_distance += calc_route_distance(inst, &route);
    }

    Ok(total_distance)
}

