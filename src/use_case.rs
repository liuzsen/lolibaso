use crate::result::BizResult;

pub mod long_run;

pub trait UseCase: 'static {
    type Input;
    type Output;
    type Error;

    fn execute(
        self,
        input: Self::Input,
    ) -> impl Future<Output = BizResult<Self::Output, Self::Error>> + 'static;
}
