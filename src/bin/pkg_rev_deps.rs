use std::env::args;

use pkg::{package::PackageError, recipes::ReverseDependencies};

fn main() -> Result<(), PackageError> {
    let query = args()
        .nth(1)
        .map(|arg| ReverseDependencies::query(&arg))
        .unwrap()?;

    for dep in query.rev_deps {
        println!("{}", dep.name);
    }

    Ok(())
}
