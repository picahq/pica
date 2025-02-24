use clap::{
    CommandFactory,
    builder::StyledStr,
    error::{ContextKind, ContextValue, ErrorFormatter, ErrorKind, RichFormatter},
};
use entities::Unit;
use std::fmt::Write;
use std::io::Write as IoWrite;

#[derive(Debug, Clone, Copy)]
pub struct Printer;

impl Printer {
    pub fn stderr<T>(
        &self,
        error: &str,
        kind: ErrorKind,
        sugg: impl Into<Option<&'static str>>,
        is_fatal: bool,
    ) -> Unit
    where
        T: CommandFactory,
    {
        let mut cmd = T::command();
        let mut err = cmd.error(kind, error).apply::<RichFormatter>();

        if let Some(sugg) = sugg.into() {
            let mut suggestion = StyledStr::new();
            suggestion.write_str(sugg).expect(sugg);

            err.insert(
                ContextKind::Suggested,
                ContextValue::StyledStrs(vec![suggestion]),
            );
        }

        let s = RichFormatter::format_error(&err);
        eprintln!("{}", s.ansi());
        eprintln!("-----");

        if is_fatal {
            err.exit()
        } else {
            err.print().expect(error)
        }
    }

    pub fn stdout(&self, message: &str) -> Unit {
        println!("{}", message);
    }

    pub fn write(&self, message: &str) -> Unit {
        write!(std::io::stdout(), "{}", message).expect(message);
        std::io::stdout().flush().expect(message);
    }
}
