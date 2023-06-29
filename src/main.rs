#![feature(bigint_helper_methods)]
mod engine;
mod io;
mod types;
mod utils;


fn main() {
    let filename = "./Example.class";
    let class = io::load_class_from(filename);

    // Spin the jvm and try to load the class.
    let mut runtime = engine::Runtime::new();
    let result = runtime.check_if_can_run(&class);
    if !result.is_ok() {

        println!("{}", result.err().unwrap())
    }

    runtime.entrypoint(filename.split(".").next().unwrap(), &class);
}
