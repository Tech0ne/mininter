#[derive(Clone, Debug)]
pub struct DebugState {
    file: &'static str,
    function: &'static str,
    line: u32,
    column_start: u16,
    column_end: u16,
}

pub enum DebugDelta {
    Full(DebugState),

    // only line changed
    Line {
        line: u32,
    },

    // only columns changed
    Columns {
        start: u16,
        end: u16,
    },

    // line + columns
    LineColumns {
        line: u32,
        start: u16,
        end: u16,
    },

    // entered another function
    Function {
        function: &'static str,
    },

    // changed file
    File {
        file: &'static str,
    },

    // arbitrary combination
    Patch {
        file: Option<&'static str>,
        function: Option<&'static str>,
        line: Option<u32>,
        start: Option<u16>,
        end: Option<u16>,
    },
}

impl DebugDelta {
    pub fn apply(&self, state: &mut DebugState) {
        match self {
            DebugDelta::Full(s) => *state = s.clone(),

            DebugDelta::Line { line } => {
                state.line = *line;
            }

            DebugDelta::Columns { start, end } => {
                state.column_start = *start;
                state.column_end = *end;
            }

            DebugDelta::LineColumns { line, start, end } => {
                state.line = *line;
                state.column_start = *start;
                state.column_end = *end;
            }

            DebugDelta::Function { function } => {
                state.function = function;
            }

            DebugDelta::File { file } => {
                state.file = file;
            }

            DebugDelta::Patch {
                file,
                function,
                line,
                start,
                end,
            } => {
                if let Some(v) = file {
                    state.file = v;
                }
                if let Some(v) = function {
                    state.function = v;
                }
                if let Some(v) = line {
                    state.line = *v;
                }
                if let Some(v) = start {
                    state.column_start = *v;
                }
                if let Some(v) = end {
                    state.column_end = *v;
                }
            }
        }
    }
}
