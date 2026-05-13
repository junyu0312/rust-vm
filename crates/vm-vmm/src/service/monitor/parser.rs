use winnow::Parser;
use winnow::ascii::multispace1;
use winnow::combinator::alt;
use winnow::combinator::preceded;
use winnow::token::take_till;

use crate::service::monitor::command::MonitorCommand;

fn parse_pause(input: &mut &str) -> winnow::Result<MonitorCommand> {
    "pause".map(|_| MonitorCommand::Pause).parse_next(input)
}

fn parse_resume(input: &mut &str) -> winnow::Result<MonitorCommand> {
    "resume".map(|_| MonitorCommand::Resume).parse_next(input)
}

fn parse_save(input: &mut &str) -> winnow::Result<MonitorCommand> {
    preceded(("save", multispace1), take_till(1.., |_| false))
        .map(str::trim)
        .map(|path| MonitorCommand::Save(path.into()))
        .parse_next(input)
}

impl TryFrom<&str> for MonitorCommand {
    type Error = winnow::error::ContextError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let mut input = input;

        alt((parse_pause, parse_resume, parse_save)).parse_next(&mut input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        {
            let input = "pause";
            assert_eq!(MonitorCommand::try_from(input), Ok(MonitorCommand::Pause));
        }

        {
            let input = "resume";
            assert_eq!(MonitorCommand::try_from(input), Ok(MonitorCommand::Resume));
        }

        {
            let input = "save ./snapshot";
            assert_eq!(
                MonitorCommand::try_from(input),
                Ok(MonitorCommand::Save("./snapshot".into()))
            );
        }
    }
}
