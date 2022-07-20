use crate::module::Module;

pub trait Plugin {
    fn build(module: &mut Module);
}
