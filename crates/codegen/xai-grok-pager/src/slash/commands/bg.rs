//! `/bg`: branch the current session into a peer top-level agent and stay here.
//!
//! The command reuses `/fork`'s flag parsing but forces `background: true`.
//! It returns [`Action::Fork`](crate::app::actions::Action::Fork) with `background` set.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand, slash_meta};
use super::fork::parse_fork_args;

pub struct BgCommand;

impl SlashCommand for BgCommand {
    slash_meta! {
        name: "bg",
        description: "Fork this session and stay here",
        usage: "/bg [--worktree|--no-worktree] [directive]",
        takes_args: true,
        args_required: false,
        session_scoped: true,
        arg_placeholder: "[directive]",
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        match parse_fork_args(args) {
            Ok(mut parsed) => {
                parsed.background = true; // Force background mode
                CommandResult::Action(Action::Fork(parsed))
            }
            Err(msg) => CommandResult::Error(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;

    fn make_ctx(models: &ModelState) -> CommandExecCtx<'_> {
        let bundle = Box::leak(Box::new(crate::app::bundle::BundleState::default()));
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: bundle,
            screen_mode: crate::app::ScreenMode::Inline,
            billing_surface_visible: true,
            usage_command_visible: true,
            pager_state: crate::settings::PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..crate::settings::PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn run_no_args_returns_fork_action_with_background_true() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = BgCommand;
        match cmd.run(&mut ctx, "") {
            CommandResult::Action(Action::Fork(args)) => {
                assert_eq!(args.worktree_override, None);
                assert_eq!(args.directive, None);
                assert!(args.background, "/bg must force background=true");
            }
            other => panic!("expected Action(Fork(..)), got {other:?}"),
        }
    }

    #[test]
    fn run_worktree_with_directive_returns_action_with_background_true() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = BgCommand;
        match cmd.run(&mut ctx, "--worktree investigate bug") {
            CommandResult::Action(Action::Fork(args)) => {
                assert_eq!(args.worktree_override, Some(true));
                assert_eq!(args.directive.as_deref(), Some("investigate bug"));
                assert!(args.background, "/bg must force background=true");
            }
            other => panic!("expected Action(Fork(..)), got {other:?}"),
        }
    }

    #[test]
    fn run_no_worktree_with_directive_returns_action_with_background_true() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = BgCommand;
        match cmd.run(&mut ctx, "--no-worktree quick fix") {
            CommandResult::Action(Action::Fork(args)) => {
                assert_eq!(args.worktree_override, Some(false));
                assert_eq!(args.directive.as_deref(), Some("quick fix"));
                assert!(args.background, "/bg must force background=true");
            }
            other => panic!("expected Action(Fork(..)), got {other:?}"),
        }
    }

    #[test]
    fn run_background_flag_is_overridden_to_true() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = BgCommand;
        // Even if the user types --background, it's still forced to true
        match cmd.run(&mut ctx, "--background test") {
            CommandResult::Action(Action::Fork(args)) => {
                assert_eq!(args.directive.as_deref(), Some("test"));
                assert!(args.background, "/bg must force background=true");
            }
            other => panic!("expected Action(Fork(..)), got {other:?}"),
        }
    }

    #[test]
    fn run_conflicting_flags_returns_error_result() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = BgCommand;
        match cmd.run(&mut ctx, "--worktree --no-worktree") {
            CommandResult::Error(msg) => {
                assert!(msg.contains("mutually exclusive"), "got: {msg}");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn metadata_matches_design() {
        let cmd = BgCommand;
        assert_eq!(cmd.name(), "bg");
        assert!(cmd.takes_args(), "/bg accepts args");
        assert!(!cmd.args_required(), "/bg allows no args");
        assert_eq!(cmd.arg_placeholder(), Some("[directive]"));
    }
}
