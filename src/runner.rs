/// A runner is a trait that defines a single benchmark function and associated parameter types.
/// The trait is implemented on the data structure that is to be benchmarked, and the [`execute`] method
/// takes an additional parameter that is passed to the benchmark function.
///
/// The trait is implemented using the [`runner!`] macro.
pub(crate) trait Runner {
    type Context;

    type Param;

    fn create_context(&self, size: usize) -> Self::Context;

    fn prepare_params(&self, number: usize, size: usize) -> Box<[Self::Param]>;

    fn execute(&self, context: &Self::Context, param: &Self::Param);
}

#[macro_export]
macro_rules! runner {
    ($name:ident, create_context = |$size:ident| { $($ctx_body:tt)* }, prepare_params = |$number:ident, $size_params:ident| { $($param_body:tt)* }, execute = |$context:ident: $context_type:ty, $param:ident: $param_type:ty| { $($body:tt)* }) => {
        pub(crate) struct $name;

        impl runner::Runner for $name {
            type Context = $context_type;

            type Param = $param_type;

            fn create_context(&self, $size: usize) -> Self::Context {
                $($ctx_body)*
            }

            fn prepare_params(&self, $number: usize, $size_params: usize) -> Box<[Self::Param]> {
                $($param_body)*
            }

            fn execute(&self, $context: &Self::Context, $param: &Self::Param) {
                $($body)*
            }
        }
    }
}