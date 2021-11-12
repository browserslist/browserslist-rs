use browserslist::Opts;
use util::run_compare;

mod util;

#[test]
fn firefox_esr() {
    let opts = Opts::new();
    run_compare("firefox esr", &opts);
    run_compare("Firefox ESR", &opts);
    run_compare("ff esr", &opts);
    run_compare("Ff ESR", &opts);
    run_compare("fx esr", &opts);
    run_compare("Fx ESR", &opts);
}
