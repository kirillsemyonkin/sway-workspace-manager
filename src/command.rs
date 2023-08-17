#[derive(Clone, Copy)]
pub enum Position {
    Prev { cycle: bool },
    Next { cycle: bool, extra: bool },
    Start,
    End { extra: bool },
    Num { num: usize, extra: bool },
}

impl Position {
    pub fn num_existing(
        self,
        current_index: usize,
        num_workspaces: usize,
    ) -> Result<usize, swayipc::Error> {
        let (index, len) = (current_index, num_workspaces);

        use Position::*;
        match self {
            Prev { cycle } => {
                if index != 1 {
                    Ok(index - 1)
                } else if cycle {
                    Ok(len)
                } else {
                    Err(swayipc::Error::CommandFailed(
                        "No previous workspace in the first workspace".to_string(),
                    ))
                }
            }
            Next { cycle, extra } => {
                if index != len {
                    Ok(index + 1)
                } else if cycle {
                    Ok(1)
                } else if extra {
                    Ok(index + 1)
                } else {
                    Err(swayipc::Error::CommandFailed(
                        "No next workspace in the last workspace".to_string(),
                    ))
                }
            }
            Start => Ok(1),
            End { extra: true } => Ok(len + 1),
            End { extra: false } => Ok(len),
            Num { num, extra } => {
                if 1 <= num && (!extra && num <= len || extra && num <= len + 1) {
                    Ok(num)
                } else {
                    Err(swayipc::Error::CommandFailed(
                        "Workspace number out of range".to_string(),
                    ))
                }
            }
        }
    }

    pub fn num_new(
        self,
        current_index: usize,
        num_workspaces: usize,
    ) -> Result<usize, swayipc::Error> {
        let (index, len) = (current_index, num_workspaces);

        use Position::*;
        match self {
            Prev { .. } => Ok(index),
            Next { .. } => Ok(index + 1),
            Start => Ok(1),
            End => Ok(len + 1),
            Num { num, .. } => {
                if 1 <= num && num <= len + 1 {
                    Ok(num)
                } else {
                    Err(swayipc::Error::CommandFailed(
                        "Workspace number out of range".to_string(),
                    ))
                }
            }
        }
    }
}

pub enum Command {
    Reorder { daemon: bool },
    Switch { target: Position, carry: bool },
    Create { target: Position, carry: bool },
    Swap { target: Position },
    Rename { new_name: String },
}

impl Command {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        use Command::*;

        let _cmd_alias = args.next();

        let verb = args.next().ok_or("not enough arguments")?;

        if verb.as_str() == "reorder" {
            let daemon = args.any(|flag| flag.as_str() == "--daemon");
            return Ok(Reorder { daemon });
        }

        if verb.as_str() == "rename" {
            let new_name = args.next().ok_or("not enough arguments")?;
            return Ok(Rename { new_name });
        }

        let position = args.next().ok_or("not enough arguments")?;

        let mut cycle = false;
        let mut extra = false;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--cycle" => cycle = true,
                "--extra" => extra = true,
                _ => {}
            };
        }

        use Position::*;
        let target = match position.as_str() {
            "prev" => Prev { cycle },
            "next" => Next { cycle, extra },
            "start" => Start,
            "end" => End { extra },
            other => other
                .parse::<usize>()
                .map(|num| Num { num, extra })
                .or(Err("invalid target"))?,
        };

        Ok(match verb.as_str() {
            "switch" => Switch {
                target,
                carry: false,
            },
            "move" => Switch {
                target,
                carry: true,
            },
            "create" => Create {
                target,
                carry: false,
            },
            "move-to-new" => Create {
                target,
                carry: true,
            },
            "swap" => Swap { target },
            _ => Err("invalid command")?,
        })
    }
}
