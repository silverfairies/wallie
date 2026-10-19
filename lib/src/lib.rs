pub mod communicate;
pub mod dataset;
pub mod rules;

#[cfg(debug_assertions)]
pub fn dprint(print: String) {
    eprintln!("{}", print);
}

#[cfg(not(debug_assertions))]
pub fn dprint(_print: String) {
    ()
}
