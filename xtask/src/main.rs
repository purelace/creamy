#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

mod test;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Test {
        #[arg(short, long)]
        project: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Test { project } => {
            if let Some(project) = project {
                test::run_project_tests(&project)
            } else {
                test::run_all_tests()
            }
        }
    }
}
