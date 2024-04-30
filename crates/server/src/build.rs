use crate::component_def::Build;
use std::{env, fs::remove_dir_all, io, process::Command};

pub struct GitBuild {}

impl Build for GitBuild {
    fn build(&self, repo: String) -> Result<(), io::Error> {
        let split: Vec<&str> = repo.split(" ").collect();

        println!("split: {:?}", split);

        let repo = split[0].to_string();

        let repo_name = repo.clone().split("/").last().unwrap().to_string();
        let repo_name = repo_name[0..repo_name.len() - 4].to_string(); // remove .git from end

        //saving current directory
        let current_dir = env::current_dir().unwrap();

        // Clone repo using git
        try_command(
            Command::new("git").arg("clone").arg(repo),
            "failed to clone repo",
        )?;

        // Change directory to the repo
        env::set_current_dir(repo_name.clone()).unwrap();

        // switching branch if specified
        if split.len() > 1 {
            let branch = split[1].to_string();
            // Checkout to the branch
            try_command(
                Command::new("git").arg("checkout").arg(branch),
                "failed to checkout branch",
            )?;
        }

        // run script (run with bash for linux and powershell for windows)
        println!("starting process {}", repo_name);
        if cfg!(target_os = "windows") {
            try_command(
                Command::new("powershell").arg("./run.ps1"),
                "failed to execute process",
            )?;
        } else {
            try_command(
                Command::new("bash").arg("run.sh"),
                "failed to execute process",
            )?;
        };
        println!("process finished");

        // going back to the original directory
        env::set_current_dir(current_dir).unwrap();

        // Removing the cloned repo
        remove_dir_all(repo_name).unwrap();

        Ok(())
    }
}

fn try_command(command: &mut Command, error_message: &str) -> Result<(), io::Error> {
    let output = command.output().expect(error_message);

    if output.stderr.len() > 0 {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    if output.stdout.len() > 0 {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}
