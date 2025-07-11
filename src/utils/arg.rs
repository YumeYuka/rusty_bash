//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub fn consume_option(opt: &str, args: &mut Vec<String>) -> bool {
    let end = args.iter().position(|a| a == "--");

    match args.iter().position(|a| a == opt) {
        Some(pos) => {
            if let Some(e) = end {
                if e < pos {
                    return false;
                }
            }

            args.remove(pos);
            true
        },
        None => false,
    }
}

pub fn consume_starts_with(s: &str, args: &mut Vec<String>) -> Vec<String> {
    let mut ans = args.clone();
    ans.retain(|a| a.starts_with(s));
    args.retain(|a| ! a.starts_with(s));
    ans
}

pub fn consume_with_next_arg(prev_opt: &str, args: &mut Vec<String>) -> Option<String> {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            match pos+1 >= args.len() {
                true  => None,
                false => {
                    args.remove(pos);
                    Some(args.remove(pos))
                },
            }
        },
        None => None,
    }
}

pub fn consume_with_subsequents(prev_opt: &str, args: &mut Vec<String>) -> Vec<String> {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            let ans = args[pos..].to_vec();
            *args = args[..pos].to_vec();
            ans
        },
        None => vec![],
    }
}

fn dissolve_option(opt: &str) -> Vec<String> {
    if opt.starts_with("-") {
        opt[1..].chars().map(|c| ("-".to_owned() + &c.to_string()).to_string()).collect()
    }else if opt.starts_with("+") {
        opt[1..].chars().map(|c| ("+".to_owned() + &c.to_string()).to_string()).collect()
    }else {
        vec![opt.to_string()]
    }
}

pub fn dissolve_options(args: &Vec<String>) -> Vec<String> {
    //args.iter().map(|a| dissolve_option(a)).collect::<Vec<Vec<String>>>().concat()

    let mut ans = vec![];
    let mut after_double_hyphen = false;
    for a in args {
        if a == "--" {
            after_double_hyphen = true;
        }

        match after_double_hyphen {
            true => ans.push(a.to_string()),
            false => ans.append(&mut dissolve_option(a)),
        }
    }

    ans
}
