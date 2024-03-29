use crate::component_def::Build;
use std::env;
use std::fs::remove_dir_all;
use std::process::Command;

pub struct BuilderImplem {}

impl Build for BuilderImplem {
    fn build(&self, repo: String) -> bool {
        let repo_name = repo.clone().split("/").last().unwrap().to_string();
        let repo_name = repo_name[0..repo_name.len() - 4].to_string(); // remove .git from end

        //saving current directory
        let current_dir = env::current_dir().unwrap();

        // Clone repo using git
        Command::new("git")
            .arg("clone")
            .arg(repo)
            .output()
            .expect("failed to clone repo");

        // Change directory to the repo
        env::set_current_dir(repo_name.clone()).unwrap();

        // run script (run with bash for linux and powershell for windows)
        println!("starting process {}...", repo_name);
        if cfg!(target_os = "windows") {
            Command::new("powershell")
                .arg("./run.ps1")
                .output()
                .expect("failed to execute process");
        } else {
            Command::new("bash")
                .arg("run.sh")
                .output()
                .expect("failed to execute process");
        };
        println!("process finished");

        // going back to the original directory
        env::set_current_dir(current_dir).unwrap();

        // Removing the cloned repo
        remove_dir_all(repo_name).unwrap();

        true
    }
}
