use crate::component_def::Build;

pub struct BuilderImplem {}

impl Build for BuilderImplem {
    fn build(&self, build_script: String) -> bool {
        println!("building not implemented");
        return false;
    }
}
