// Copyright 2020-2024 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::io::Write as _;

use clap::builder::NonEmptyStringValueParser;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::RefTarget;
use jj_lib::str_util::StringPattern;

use crate::cli_util::{parse_string_pattern, user_error, CommandError, CommandHelper, RevisionArg};
use crate::ui::Ui;

/// Manage tags.
#[derive(clap::Subcommand, Clone, Debug)]
pub enum TagCommand {
    #[command(visible_alias("c"))]
    Create(TagCreateArgs),
    #[command(visible_alias("l"))]
    List(TagListArgs),
}

/// Create a new tag.
#[derive(clap::Args, Clone, Debug)]
pub struct TagCreateArgs {
    /// The tag's target revision.
    #[arg(long, short)]
    revision: Option<RevisionArg>,

    /// The tag to create.
    #[arg(required = true, value_parser=NonEmptyStringValueParser::new())]
    name: String,
}

/// List tags.
#[derive(clap::Args, Clone, Debug)]
pub struct TagListArgs {
    /// Show tags whose local name matches
    ///
    /// By default, the specified name matches exactly. Use `glob:` prefix to
    /// select tags by wildcard pattern. For details, see
    /// https://github.com/martinvonz/jj/blob/main/docs/revsets.md#string-patterns.
    #[arg(value_parser = parse_string_pattern)]
    pub names: Vec<StringPattern>,
}

pub fn cmd_tag(
    ui: &mut Ui,
    command: &CommandHelper,
    subcommand: &TagCommand,
) -> Result<(), CommandError> {
    match subcommand {
        TagCommand::Create(sub_args) => cmd_tag_create(ui, command, sub_args),
        TagCommand::List(sub_args) => cmd_tag_list(ui, command, sub_args),
    }
}

fn cmd_tag_create(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &TagCreateArgs,
) -> Result<(), CommandError> {
    let mut workspace_command = command.workspace_helper(ui)?;
    let target_commit =
        workspace_command.resolve_single_rev(args.revision.as_deref().unwrap_or("@"), ui)?;
    let view = workspace_command.repo().view();

    if  view.get_tag(&args.name).is_present(){
        return Err(user_error(
            format!("Tag already exists: {}", args.name)
        ));
    }

    let mut tx = workspace_command.start_transaction();

    tx.mut_repo()
        .set_tag_target(&args.name, RefTarget::normal(target_commit.id().clone()));

    tx.finish(
        ui,
        format!(
            "create {} pointing to commit {}",
            args.name,
            target_commit.id().hex()
        ),
    )?;
    Ok(())
}

fn cmd_tag_list(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &TagListArgs,
) -> Result<(), CommandError> {
    let workspace_command = command.workspace_helper(ui)?;
    let repo = workspace_command.repo();
    let view = repo.view();

    ui.request_pager();
    let mut formatter = ui.stdout_formatter();
    let formatter = formatter.as_mut();

    for name in view.tags().keys() {
        if !args.names.is_empty() && !args.names.iter().any(|pattern| pattern.matches(name)) {
            continue;
        }

        writeln!(formatter.labeled("tag"), "{name}")?;
    }

    Ok(())
}
