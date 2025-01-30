#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;

pub trait IteratorExt: Iterator {
    /// Split the iterator into two parts when the predicate first time turns
    /// true.
    ///
    /// The first part is the part before the predicate is true.
    /// The second part is the part after the predicate is true.
    fn split_when(
        self,
        predicate: impl Fn(&Self::Item) -> bool,
    ) -> (impl Iterator<Item = Self::Item>, Self)
    where
        Self: Sized;
}

impl<I: Iterator> IteratorExt for I {
    fn split_when(
        mut self,
        predicate: impl Fn(&Self::Item) -> bool,
    ) -> (impl Iterator<Item = Self::Item>, I)
    where
        Self: Sized,
    {
        let mut v = Vec::new();
        for item in self.by_ref() {
            if predicate(&item) {
                break;
            }
            v.push(item);
        }
        (v.into_iter(), self)
    }
}

/// Parse a command into an executable and arguments.
///
/// Note that the returned executable string will not contains leading and
/// suffix quotes.
pub fn parse_command(command: impl AsRef<str>) -> (String, String) {
    let mut command = command.as_ref().trim().chars().peekable();
    if ['\'', '"'].contains(command.peek().expect("command is empty")) {
        let first_quote = command.next().unwrap();
        let (executable, rest) = command.split_when(|c| c == &first_quote);
        let args = rest.skip_while(|c| *c == ' ').collect::<String>();
        (executable.collect::<String>(), args)
    } else {
        let (executable, args) = command.split_when(|c| *c == ' ');
        (executable.collect::<String>(), args.collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_when() {
        let v = vec![1, 2, 3, 4, 5];
        let (a, b) = v.into_iter().split_when(|x| *x == 3);
        assert_eq!(a.collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(b.collect::<Vec<_>>(), vec![4, 5]);
    }

    #[test]
    fn test_parse_command() {
        // Test with no quotes around the executable
        let command = r#"ppp arg1 "'arg 2 with spaces'""#;
        let (executable, args) = parse_command(command);
        assert_eq!(executable, "ppp");
        assert_eq!(args, "arg1 \"'arg 2 with spaces'\"");

        // Test with single quotes around the executable
        let command = r#"'C:\Program Files\My App\myapp.exe' arg1 'arg 2 with spaces'"#;
        let (executable, args) = parse_command(command);
        assert_eq!(executable, "C:\\Program Files\\My App\\myapp.exe");
        assert_eq!(args, "arg1 'arg 2 with spaces'");

        // Test with double quotes around the executable
        let command = r#""C:\Program Files\My App\myapp.exe" arg1 'arg 2 with spaces'"#;
        let (executable, args) = parse_command(command);
        assert_eq!(executable, "C:\\Program Files\\My App\\myapp.exe");
        assert_eq!(args, "arg1 'arg 2 with spaces'");
    }
}
