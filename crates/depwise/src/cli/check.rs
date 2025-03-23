use crate::cli::CheckArgs;

pub fn execute(check_args: CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Checking dependencies for {}",
        check_args.path.to_string_lossy()
    );

    let environment = match check_args.environment {
        env if env.current_environment => None,
        env if env.pyproject.is_some() => env
            .pyproject
            .map(depwise_analysis::EnvironmentBuilderSource::PyProjectToml),
        env if env.requirements.is_some() => env
            .requirements
            .map(depwise_analysis::EnvironmentBuilderSource::RequirementsTxt),
        env if env.condayml.is_some() => env
            .condayml
            .map(depwise_analysis::EnvironmentBuilderSource::CondaEnvironmentYml),
        _ => None,
    };

    let analysis =
        depwise_analysis::analyze_project(environment, check_args.backend.into(), &check_args.path);

    Ok(())
}
