extern crate cc;

fn main() {
    cc::Build::new()
        .include("c")
        .include(&format!("arch/{}/c", std::env::var("TARGET").unwrap().split('-').next().unwrap()))
        .files(["term"].into_iter()
                   .map(|name| format!("c/{}.c", name)))
        .compile("cursebox");
}
