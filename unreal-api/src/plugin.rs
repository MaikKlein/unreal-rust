use crate::module::Module;

pub trait Plugin {
    fn build(&self, module: &mut Module);
}
