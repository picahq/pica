use osentities::{PicaError, Unit};

// Doesn't need a State for now since each command is atomic
pub trait Handler<Context, Command, Event> {
    fn load(&self) -> impl Future<Output = Result<Context, PicaError>>;

    fn validate(&self, ctx: &Context) -> impl Future<Output = Result<Unit, PicaError>>;

    // Consider adding the command in case some preprocess is needed
    fn run(&self, ctx: &Context) -> impl Future<Output = Result<Unit, PicaError>>;

    // fn callback<Fut, T>(
    //     f: impl for<'a> Fn(&'a u8) -> Fut,
    //     ctx: &State,
    //     prer: T,
    // ) -> impl Future<Output = Result<Unit, PicaError>>
    // where
    //     Fut: Future<Output = Result<Unit, PicaError>> + Send;
}
