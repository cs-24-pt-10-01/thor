use crate::component_def::Build;

pub struct defBuild{}

impl Build for defBuild{
    fn build(&self, build_script: String) -> bool{
        println!("building not implemented");
        return false;
    }
}

