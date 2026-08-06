macro_rules! errors {
    ($($name:ident),* $(,)?) => {
        paste::paste! {
            #[derive(Debug, Default)]
            pub enum ErrorKind {
                #[default]
                Raw,
                $(
                    [<$name Error>],
                )*
            }

            impl std::fmt::Display for ErrorKind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }

            #[derive(Debug, Default)]
            pub struct LineInfos {
                line_nb: usize,
                line: String,
                span: (usize, usize),
            }

            #[derive(Debug, Default)]
            pub struct Error {
                kind: ErrorKind,
                msg: String,
                stack_trace: Vec<LineInfos>,
            }

            impl std::fmt::Display for Error {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    writeln!(f, "Error: {}: {}", self.kind, self.msg)?;

                    for trace in &self.stack_trace {
                        writeln!(f, " {:<4} | {}", trace.line_nb, trace.line)?;
                        writeln!(
                            f,
                            "{}{}{}",
                            " ".repeat(trace.span.0 + 8),
                            '^',
                            "~".repeat(trace.span.1 - 1)
                        )?;
                    }

                    Ok(())
                }
            }

            impl Error {
                pub fn line_infos(line_nb: usize, line: impl Into<String>, span: (usize, usize)) -> Option<LineInfos> {
                    Some(LineInfos{
                        line_nb,
                        line: line.into(),
                        span,
                    })
                }

                $(
                    pub fn [<$name:snake>] (msg: impl Into<String>, line_infos: Option<LineInfos>) -> Self {
                        Error {
                            kind: ErrorKind::[<$name Error>],
                            msg: msg.into(),
                            stack_trace: if let Some(line_infos) = line_infos {
                                std::iter::once(line_infos).collect::<Vec<_>>()
                            } else {
                                Vec::new()
                            },
                        }
                    }
                )*
            }
        }
    };
}

pub(super) use errors;
