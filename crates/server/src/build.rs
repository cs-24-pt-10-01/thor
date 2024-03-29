use crate::component_def::Build;
use std::{process::Command, string};

pub struct BuilderImplem {}

impl Build for BuilderImplem {
    fn build(&self, repo: String) -> bool {
        let repo_name = repo.clone().split("/").last().unwrap().to_string();
        let repo_name = repo_name[0..repo_name.len() - 4].to_string(); // remove .git from end
        let repo_name_clone = repo_name.clone();

        // clone repo using git
        Command::new("git")
            .arg("clone")
            .arg(repo)
            .output()
            .expect("failed to clone repo");

        //TODO change env directory

        // run script (run with bash for linux and powershell for windows)
        println!("starting process");
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

        Command::new("cd")
            .arg("..")
            .output()
            .expect("failed to move of folder??");

        // clean up
        Command::new("rm")
            .arg("-rf")
            .arg(repo_name_clone)
            .output()
            .expect("failed to remove repo");

        true
    }
}
