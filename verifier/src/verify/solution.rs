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
        write!(f, "Instance name: {}\n", self.instance_name.to_uppercase())?;
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
                    let mut s = r.as_span().as_str().to_owned().to_lowercase();
                    s.retain(|c| !c.is_whitespace());
                    instance_name = s;
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
            "Route 4: 8 9 10 11 12\n"
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
            "Route 4: 8 9 10 11 12\n"
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "lr2_8_1".to_string(),
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
    fn save_solution() {
        let sol = Solution {
            instance_name: "LC1_8_7".to_string(),
            routes: vec![vec![7, 8], vec![9, 10, 11], vec![5, 4, 3, 2, 1], vec![6]],
        };
        let today = chrono::Local::now().format("%Y-%m-%d");
        assert_eq!(
            sol.to_string(),
            format!(
            "Instance name: LC1_8_7\nAuthors: \nDate: {today}\nReference: \nSolution\nRoute 1: 7 8\nRoute 2: 9 10 11\nRoute 3: 5 4 3 2 1\nRoute 4: 6\n"
            )
        );
    }

    #[test]
    fn whitespaces_are_ok() {
        let sol_str = concat!(
            "Instance name\t:  \t   rc1_4_10\n",
            "Authors : \n",
            "Date:\n",
            "Reference    : \n",
            "Solution\n",
            "Route 1: 1   2     3\n",
            "Route    2   : 4\t5 6\n",
            "Route\t 3: 7\n",
            "Route 4: 8 9\t10     11\t12         \n"
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
    fn nonsequential_routes() {
        let sol_str = concat!(
            "Instance name: rc1_4_10\n",
            "Authors: \n",
            "Date:\n",
            "Reference: \n",
            "Solution\n",
            "Route 0: 1 2 3\n",
            "Route 7 : 4 5 6\n",
            "Route 3: 7\n",
            "Route: 8 9 10 11 12\n"
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
    fn whitespace_in_instance_name() {
        let sol_str = concat!(
            "Instance name: rc 1 _ \t 4_10\n",
            "Authors: \n",
            "Date:\n",
            "Reference: \n",
            "Solution\n",
            "Route 0: 1 2 3\n",
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "rc1_4_10".to_string(),
                routes: vec![vec![1, 2, 3],],
            }
        );
    }

    #[test]
    fn no_instance_name() {
        let sol_str = concat!(
            "Instance name:\n",
            "Authors: \n",
            "Date:\n",
            "Reference: \n",
            "Solution\n",
            "Route 0: 1 2 3\n",
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "".to_string(),
                routes: vec![vec![1, 2, 3],],
            }
        );
    }

    #[test]
    fn whatever_in_other_fields() {
        let sol_str = concat!(
            "Instance name:\n",
            "Authors: my pet hamster\n",
            "Date: whatever 2023-13-72\n",
            "Reference: 中文范例文本نص مثال عربي\n",
            "Solution\n",
            "Route 0: 1 2 3\n",
        );

        let sol = Solution::from_str(sol_str);

        assert_eq!(sol.as_ref().err(), None);

        assert_eq!(
            sol.unwrap(),
            Solution {
                instance_name: "".to_string(),
                routes: vec![vec![1, 2, 3],],
            }
        );
    }

    #[test]
    fn good_instance_names() {
        for pdp in ["", "l"] {
            for t in ["c1", "c2", "r1", "r2", "rc1", "rc2"] {
                for s in (2..=10).step_by(2) {
                    for i in 1..=10 {
                        let inst = format!("{pdp}{t}_{s}_{i}");
                        assert!(SolutionParser::parse(Rule::instance_name, &inst).is_ok());
                        let inst = format!("{pdp}{t}_{s}_{i}").to_uppercase();
                        assert!(SolutionParser::parse(Rule::instance_name, &inst).is_ok());
                    }
                }
            }
        }
    }

    #[test]
    fn bad_instance_names() {
        assert!(SolutionParser::parse(Rule::instance_name, "l").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "rc").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "lrc").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r_2_10").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r1_1_10").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "m1_1_10").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r1_m_10").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r1_2_xyz").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r1_5_10").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r1_2_").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "r2210").is_err());
        assert!(SolutionParser::parse(Rule::instance_name, "lr2210").is_err());

        assert!(SolutionParser::parse(Rule::instance_name, "r1_2_12").is_ok());
        // this is technically ok according to instance_name, but next will fail to parse:
        assert!(SolutionParser::parse(Rule::instance, "Instance name: r1_2_12\n").is_err());
    }
}
