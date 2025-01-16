use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
// I need it for chunks
use itertools::{izip, Itertools};
use std::{error::Error, str::FromStr};

const LINE_WIDTH: usize = 22;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("Someone <someone@mail.com>")
        .about("Rust cal")
        // what goes here?
        .arg(
            Arg::with_name("show_current_year")
                .short("y")
                .long("year")
                .help("Shows whole current year")
                .takes_value(false)
                .conflicts_with_all(&["year", "month"]),
        )
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .help("Month name or number (1-12)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1-9999)"),
        )
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;
    let today = Local::now();

    if matches.is_present("show_current_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.date_naive(),
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(num) if (1..=12).contains(&num) => Ok(num),
        Ok(num) => Err(format!("month \"{}\" not in the range 1 through 12", num).into()),
        _ => {
            let lower = &month.to_lowercase();
            let matches = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(p, m)| {
                    if m.to_lowercase().starts_with(lower) {
                        Some(p + 1)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(From::from(format!("Invalid month \"{}\"", month)))
            }
        }
    }
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", num).into())
        }
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let month_name = MONTH_NAMES[month as usize - 1];
    let header_text = if print_year {
        format!("{} {}", month_name, year)
    } else {
        month_name.to_string()
    };

    let spaces_before_1st = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .and_hms_opt(5, 0, 0)
        .unwrap()
        .weekday()
        .num_days_from_sunday();

    let is_today =
        |day: u32| -> bool { today == NaiveDate::from_ymd_opt(year, month, day).unwrap() };

    let all_cal_items = (0..42).map(|n| {
        if n < spaces_before_1st {
            "  ".to_string()
        } else if n < spaces_before_1st
            + last_day_in_month(year, month)
                .and_hms_opt(5, 0, 0)
                .unwrap()
                .day()
        {
            // highlight today here - possible day contains the day we are iterating from all_call_items
            let possible_day = n - spaces_before_1st + 1;
            if is_today(possible_day) {
                Style::new()
                    .reverse()
                    .paint(format!("{:2}", &possible_day))
                    .to_string()
            } else {
                format!("{:2}", &possible_day)
            }
        } else {
            "  ".to_string()
        }
    });

    let mut lines = Vec::with_capacity(8);
    lines.push(format!("{:^width$}  ", header_text, width = LINE_WIDTH - 2));
    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());
    lines.extend(
        all_cal_items
            .chunks(7)
            .into_iter()
            .map(|mut chunk| format!("{:<}  ", chunk.join(" "))),
    );

    // for row in &all_cal_items.chunks(7) {
    //     let mut row = row;
    //     lines.push(format!("{:<}  ", row.join(" ")),);
    // }

    lines
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()

    // NaiveDate::from_ymd_opt(
    //     year,
    //     month,
    //     match month {
    //         1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    //         2 => {
    //             if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
    //                 29
    //             } else {
    //                 28
    //             }
    //         }
    //         _ => 30,
    //     },
    // )
    // .unwrap()
}

pub fn run(config: Config) -> MyResult<()> {
    let print_year = config.year != config.today.and_hms_opt(5, 0, 0).unwrap().year();

    match config.month {
        Some(month) => {
            // this might be better than a for loop as only one print is used - so one I/O operation
            println!(
                "{}",
                format_month(config.year, month, print_year, config.today).join("\n")
            );
        }
        None => {
            println!("{:>32}", config.year);
            for &(m1, m2, m3) in &[(1, 2, 3), (4, 5, 6), (7, 8, 9), (10, 11, 12)] {
                // this looks worse than a for loop but only one print I/O operation is needed to print everything
                println!(
                    "{}",
                    izip!(
                        format_month(config.year, m1, false, config.today),
                        format_month(config.year, m2, false, config.today),
                        format_month(config.year, m3, false, config.today)
                    )
                    // deconstruct the tuple into col1, col2, col3 and build a String with them
                    .map(|(col1, col2, col3)| format!("{}{}{}", col1, col2, col3))
                    .join("\n")
                );

                if m3 != 12 {
                    println!()
                };
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1);
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today.unwrap()), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today.unwrap()), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7);
        assert_eq!(format_month(2021, 4, true, today.unwrap()), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        // original tests errored in use of deprecated associated function `chrono::NaiveDate::from_ymd`: use `from_ymd_opt()` instead
        // this returns an option, hence the .unwrap() at the end. Check UNsafe?
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        // year 2100 is not a leap year so should return 28 days in Feb
        assert_eq!(
            last_day_in_month(2100, 2),
            NaiveDate::from_ymd_opt(2100, 2, 28).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
