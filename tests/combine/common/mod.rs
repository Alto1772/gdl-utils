use glob::glob;

pub(crate) fn listglob(path: &str) -> Vec<String> {
    glob(path)
        .unwrap()
        .map(|p| p.unwrap().into_string().unwrap())
        .collect()
}

pub(crate) fn repeat_path(path: &str, n: usize) -> Vec<String> {
    std::iter::repeat_n(path.to_string(), n).collect()
}

pub(crate) fn map_path(paths: &[&str], parent: &str) -> Vec<String> {
    paths
        .iter()
        .map(|p| {
            if p.starts_with('+') {
                p.to_string()
            } else {
                parent.to_string() + p
            }
        })
        .collect()
}

pub(crate) fn tmp_output(path: &str) -> String {
    let dir = std::env::current_dir()
        .unwrap()
        .join("target/gdl-combine-outputs");
    std::fs::create_dir_all(&dir).unwrap();

    dir.join(path).to_string_lossy().to_string()
}

macro_rules! test_image_generate {
    ($test:ident, $ext:expr, $imgs:expr $(, $params:expr)? $(,)?) => {
        #[test]
        fn $test() {
            let images: Vec<String> = $imgs;
            let output: String =
                crate::common::tmp_output(concat!("output_", stringify!($test), $ext));

            assert_cmd::cargo::cargo_bin_cmd!("gdl")
                .arg("-v")
                .arg("combine")
                .arg("-o")
                .arg(&output)
                $(
                    .args($params)
                )?
                .args(images.iter().map(String::as_str))
                .assert()
                .success();
        }
    };
}

pub(crate) use test_image_generate;
