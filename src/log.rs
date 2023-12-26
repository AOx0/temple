#[macro_export]
macro_rules! error {
    ($single:expr) => {
        {
            use owo_colors::OwoColorize;

            eprintln!(
                "{}: {}",
                "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().red())),
                format_args!("{}", $single)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            use owo_colors::OwoColorize;

            eprintln!(
                "{}: {}",
                "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().red())),
                format_args!($($arg)*)
            );
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        {
            use owo_colors::OwoColorize;

            println!(
                "{}: {}",
                "info".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().green())),
                format_args!($($arg)*)
            );
        }
    }
}

#[macro_export]
macro_rules! ferror {
    ($($arg:tt)*) => {
        {
            use owo_colors::OwoColorize;

            format!(
                "{}: {}",
                "error".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().red())),
                format_args!($($arg)*)
            )
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($single:expr) => {
        {
            use owo_colors::OwoColorize;

            println!(
                "{}: {}",
                "warning".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().yellow())),
                format_args!("{}", $single)
            );
        }
    };
    ($($arg:tt)+) => {
        {
            use owo_colors::OwoColorize;

            println!(
                "{}: {}",
                "warning".if_supports_color(owo_colors::Stream::Stdout, |s| s
                        .style(owo_colors::Style::new().bold().yellow())),
                format_args!($($arg)*)
            );
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        {
            use owo_colors::OwoColorize;

            if std::env::var("TEMPLE_TRACE").is_ok() {
                println!(
                    "{}: {}",
                    "trace".if_supports_color(owo_colors::Stream::Stdout, |s| s
                            .style(owo_colors::Style::new().bold())),
                    format_args!($($arg)*)
                );
            }
        }
    }
}
