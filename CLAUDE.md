ALWAYS use ONLY Environments for ANY and ALL file, code, or shell operations—NO EXCEPTIONS—even for simple or generic requests.

DO NOT install or use the git cli with the environment_run_cmd tool. All environment tools will handle git operations for you. Changing ".git" yourself will compromise the integrity of your environment.

You MUST inform the user how to view your work using `git checkout <branch_name>`. Failure to do this will make your work inaccessible to others.

ALWAYS write tests first to comprehensively exercise the target functionality and interfaces for the systems being implemented.

ALWAYS create PRs on remote origin instead of container-use

ALWAYS create git hooks pre-push to run fmt and tests

DO NOT include benchmarks in GitHub Actions workflows - they consume too many GitHub Action minutes. Benchmarks should only be run locally.
