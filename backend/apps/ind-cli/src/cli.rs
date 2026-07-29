use anyhow::{Result, bail};

pub const DEFAULT_DLQ_LIMIT: i64 = 50;
pub const DEFAULT_EMBEDDINGS_REPAIR_LIMIT: i64 = 100;
pub const DEFAULT_SEARCH_REINDEX_PAGE_SIZE: u32 = 500;
pub const DEFAULT_RECOVERY_LIMIT: i64 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    JobsDlqList {
        limit: i64,
        json: bool,
    },
    JobsDlqShow {
        dead_letter_id: String,
        json: bool,
    },
    JobsDlqReplay {
        dead_letter_id: String,
        json: bool,
    },
    JobsDlqStats {
        json: bool,
    },
    JobsRecoveryList {
        status: Option<String>,
        job_type: Option<String>,
        limit: i64,
        json: bool,
    },
    IntegrityStats {
        json: bool,
    },
    SearchReindex {
        page_size: u32,
        json: bool,
    },
    EmbeddingsRepair {
        limit: i64,
        json: bool,
    },
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command> {
    let args = args.into_iter().collect::<Vec<_>>();
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        return Ok(Command::Help);
    }

    match args.as_slice() {
        [jobs, dlq, command, rest @ ..] if jobs == "jobs" && dlq == "dlq" => {
            parse_dlq_command(command, rest)
        }
        [jobs, recovery, command, rest @ ..] if jobs == "jobs" && recovery == "recovery" => {
            parse_recovery_command(command, rest)
        }
        [search, reindex, rest @ ..] if search == "search" && reindex == "reindex" => {
            parse_search_reindex(rest)
        }
        [embeddings, repair, rest @ ..] if embeddings == "embeddings" && repair == "repair" => {
            parse_embeddings_repair(rest)
        }
        [integrity, stats, rest @ ..] if integrity == "integrity" && stats == "stats" => {
            parse_integrity_stats(rest)
        }
        [other, ..] => bail!("unknown command `{other}`"),
        [] => Ok(Command::Help),
    }
}

fn parse_dlq_command(command: &str, args: &[String]) -> Result<Command> {
    match command {
        "list" => {
            let opts = parse_options(args, &[OptionSpec::Limit])?;
            Ok(Command::JobsDlqList {
                limit: opts.limit.unwrap_or(DEFAULT_DLQ_LIMIT),
                json: opts.json,
            })
        }
        "show" => {
            let (dead_letter_id, rest) = take_required_id(args, "dead_letter_id")?;
            let opts = parse_options(rest, &[])?;
            Ok(Command::JobsDlqShow {
                dead_letter_id: dead_letter_id.to_string(),
                json: opts.json,
            })
        }
        "replay" => {
            let (dead_letter_id, rest) = take_required_id(args, "dead_letter_id")?;
            let opts = parse_options(rest, &[])?;
            Ok(Command::JobsDlqReplay {
                dead_letter_id: dead_letter_id.to_string(),
                json: opts.json,
            })
        }
        "stats" => {
            let opts = parse_options(args, &[])?;
            Ok(Command::JobsDlqStats { json: opts.json })
        }
        other => bail!("unknown jobs dlq command `{other}`"),
    }
}

fn parse_recovery_command(command: &str, args: &[String]) -> Result<Command> {
    match command {
        "list" => {
            let opts = parse_options(
                args,
                &[OptionSpec::Limit, OptionSpec::Status, OptionSpec::JobType],
            )?;
            Ok(Command::JobsRecoveryList {
                status: opts.status,
                job_type: opts.job_type,
                limit: opts.limit.unwrap_or(DEFAULT_RECOVERY_LIMIT),
                json: opts.json,
            })
        }
        other => bail!("unknown jobs recovery command `{other}`"),
    }
}

fn parse_search_reindex(args: &[String]) -> Result<Command> {
    let opts = parse_options(args, &[OptionSpec::PageSize])?;
    Ok(Command::SearchReindex {
        page_size: opts.page_size.unwrap_or(DEFAULT_SEARCH_REINDEX_PAGE_SIZE),
        json: opts.json,
    })
}

fn parse_embeddings_repair(args: &[String]) -> Result<Command> {
    let opts = parse_options(args, &[OptionSpec::Limit])?;
    Ok(Command::EmbeddingsRepair {
        limit: opts.limit.unwrap_or(DEFAULT_EMBEDDINGS_REPAIR_LIMIT),
        json: opts.json,
    })
}

fn parse_integrity_stats(args: &[String]) -> Result<Command> {
    let opts = parse_options(args, &[])?;
    Ok(Command::IntegrityStats { json: opts.json })
}

fn take_required_id<'a>(args: &'a [String], name: &str) -> Result<(&'a str, &'a [String])> {
    match args {
        [id, rest @ ..] if !id.starts_with("--") => Ok((id, rest)),
        _ => bail!("missing required {name}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionSpec {
    Limit,
    PageSize,
    Status,
    JobType,
}

#[derive(Default)]
struct ParsedOptions {
    json: bool,
    limit: Option<i64>,
    page_size: Option<u32>,
    status: Option<String>,
    job_type: Option<String>,
}

fn parse_options(args: &[String], allowed: &[OptionSpec]) -> Result<ParsedOptions> {
    let mut parsed = ParsedOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                parsed.json = true;
                index += 1;
            }
            "--limit" if allowed.contains(&OptionSpec::Limit) => {
                let value = parse_i64_option(args, index, "--limit", "limit")?;
                if value <= 0 {
                    bail!("limit must be greater than 0");
                }
                parsed.limit = Some(value);
                index += 2;
            }
            "--page-size" if allowed.contains(&OptionSpec::PageSize) => {
                let value = parse_u32_option(args, index, "--page-size", "page size")?;
                if value == 0 {
                    bail!("page size must be greater than 0");
                }
                parsed.page_size = Some(value);
                index += 2;
            }
            "--status" if allowed.contains(&OptionSpec::Status) => {
                parsed.status = Some(parse_string_option(args, index, "--status")?);
                index += 2;
            }
            "--job-type" if allowed.contains(&OptionSpec::JobType) => {
                parsed.job_type = Some(parse_string_option(args, index, "--job-type")?);
                index += 2;
            }
            option if option.starts_with("--") => bail!("unknown option `{option}`"),
            value => bail!("unexpected argument `{value}`"),
        }
    }
    Ok(parsed)
}

fn parse_i64_option(args: &[String], index: usize, flag: &str, label: &str) -> Result<i64> {
    let Some(value) = args.get(index + 1) else {
        bail!("{flag} requires a value");
    };
    value
        .parse()
        .map_err(|_| anyhow::anyhow!("{label} must be an integer"))
}

fn parse_string_option(args: &[String], index: usize, flag: &str) -> Result<String> {
    match args.get(index + 1) {
        Some(value) if !value.starts_with("--") => Ok(value.clone()),
        _ => bail!("{flag} requires a value"),
    }
}

fn parse_u32_option(args: &[String], index: usize, flag: &str, label: &str) -> Result<u32> {
    let Some(value) = args.get(index + 1) else {
        bail!("{flag} requires a value");
    };
    value
        .parse()
        .map_err(|_| anyhow::anyhow!("{label} must be an integer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command> {
        parse_args(args.iter().map(|arg| (*arg).to_string()))
    }

    #[test]
    fn command_table_preserves_operational_cli_contracts() -> Result<()> {
        let cases: &[(&[&str], Command)] = &[
            (
                &["jobs", "dlq", "list", "--limit", "25", "--json"],
                Command::JobsDlqList {
                    limit: 25,
                    json: true,
                },
            ),
            (
                &["jobs", "dlq", "show", "dlj_0123456789abcdef", "--json"],
                Command::JobsDlqShow {
                    dead_letter_id: "dlj_0123456789abcdef".into(),
                    json: true,
                },
            ),
            (
                &["search", "reindex", "--page-size", "500", "--json"],
                Command::SearchReindex {
                    page_size: 500,
                    json: true,
                },
            ),
            (
                &["integrity", "stats", "--json"],
                Command::IntegrityStats { json: true },
            ),
            (
                &[
                    "jobs",
                    "recovery",
                    "list",
                    "--status",
                    "waiting",
                    "--job-type",
                    "document.ai.embed",
                    "--limit",
                    "10",
                    "--json",
                ],
                Command::JobsRecoveryList {
                    status: Some("waiting".into()),
                    job_type: Some("document.ai.embed".into()),
                    limit: 10,
                    json: true,
                },
            ),
            (
                &["jobs", "recovery", "list"],
                Command::JobsRecoveryList {
                    status: None,
                    job_type: None,
                    limit: DEFAULT_RECOVERY_LIMIT,
                    json: false,
                },
            ),
        ];
        for (args, expected) in cases {
            assert_eq!(&parse(args)?, expected, "{args:?}");
        }
        Ok(())
    }

    #[test]
    fn rejects_zero_limit() {
        let err = parse(&["embeddings", "repair", "--limit", "0"]).unwrap_err();
        assert!(err.to_string().contains("limit must be greater than 0"));
    }
}
