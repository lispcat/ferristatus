use ferristatus::{args::Args, config::Config, run_program};

macro_rules! config_from_str {
    ($body:expr) => {
        Config::new_from_contents($body.to_string())
    };
}

macro_rules! config_and_expected_err {
    ($body:expr, $expected:expr) => {
        let config = Config::new_from_contents($body.to_string());
        let err_str = config.unwrap_err().to_string();

        assert_eq!(err_str, $expected)
    };
}

macro_rules! config_ok_expected {
    ($body:expr, $expected:expr) => {
        let config = Config::new_from_contents($body.to_string());
        assert_eq!(format!("{:#?}", config), $expected)
    };
}

#[test]
fn config_1() {
    let args = Args {
        config_path: "tests/configs/config_1.yml".into(),
    };
    let a = run_program(args, Some(1));

    println!("DEBUG: {:#?}", a);
}

// #[test]
// fn config_2() {
//     config_and_expected_err!("settings:", "missing field `components`");
// }

// #[test]
// fn config_3() {
//     config_ok_expected!(
//         r#"
// settings:
// components:
// "#,
//         r#"Ok(
//     Config {
//         settings: Settings {
//             check_interval: 100,
//             default_separator: "|",
//         },
//         components: ComponentVec {
//             vec: [],
//         },
//     },
// )"#
//     );
// }

// TODO: try making more whole, complete configs and run it for one iteration (external config file (have only like 5 but have it cover all the big test cases))
