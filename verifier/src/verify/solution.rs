use chrono;
use itertools;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "verify/sintef_solution.pest"]
pub struct SolutionParser;

#[serde_with::serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Solution {
    pub instance_name: String,
    pub routes: Vec<Vec<usize>>,
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance name: {}\n", self.instance_name)?;
        write!(f, "Authors: \n")?;
        write!(f, "Date: {}\n", chrono::Local::now().format("%Y-%m-%d"))?;
        write!(f, "Reference: \n")?;
        write!(f, "Solution\n")?;
        for (i, route) in self.routes.iter().enumerate() {
            write!(
                f,
                "Route {}: {}\n",
                i + 1,
                itertools::join(route.iter().map(|x| x.to_string()), " ")
            )?;
        }
        Ok(())
    }
}

impl FromStr for Solution {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = SolutionParser::parse(Rule::file, input)
            .map_err(|x| x.to_string())?
            .next()
            .unwrap();

        let mut instance_name: String = "".to_string();

        let mut routes: Vec<Vec<usize>> = vec![];

        for r in parsed.into_inner() {
            match r.as_rule() {
                Rule::instance_name => {
                    instance_name = r.as_span().as_str().to_owned();
                }
                Rule::route => {
                    routes.push(
                        r.as_span()
                            .as_str()
                            .split_whitespace()
                            .into_iter()
                            .map(|c| c.parse().unwrap_or_default())
                            .collect(),
                    );
                }
                _ => unreachable!(),
            }
        }
        Ok(Solution {
            instance_name,
            routes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_gh_solution() {
        let sol_str = concat!(
            "Instance name: rc1_4_10\n",
            "Authors: \n",
            "Date:\n",
            "Reference: \n",
            "Solution\n",
            "Route 1: 1 2 3\n",
            "Route 2: 4 5 6\n",
            "Route 3: 7\n",
            "Route 3: 8 9 10 11 12\n"
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "rc1_4_10".to_string(),
                routes: vec![
                    vec![1, 2, 3],
                    vec![4, 5, 6],
                    vec![7],
                    vec![8, 9, 10, 11, 12],
                ],
            }
        );
    }

    #[test]
    fn read_ll_solution() {
        let sol_str = concat!(
            "Instance name: LR2_8_1\n",
            "Authors: \n",
            "Date:\n",
            "Reference: \n",
            "Solution\n",
            "Route 1: 1 2 3\n",
            "Route 2: 4 5 6\n",
            "Route 3: 7\n",
            "Route 3: 8 9 10 11 12\n"
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "LR2_8_1".to_string(),
                routes: vec![
                    vec![1, 2, 3],
                    vec![4, 5, 6],
                    vec![7],
                    vec![8, 9, 10, 11, 12],
                ],
            }
        );
    }
}
