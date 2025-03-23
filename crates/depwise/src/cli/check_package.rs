use crate::cli::CheckPackageArgs;

pub fn execute(args: CheckPackageArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Checking dependencies for {}",
        args.package.to_string_lossy()
    );
    Ok(())
}
