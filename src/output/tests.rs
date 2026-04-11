use crate::config::IconMode;
use crate::model::{
    ContainerContext, ContextInfo, Field, GitContext, MeInfo, NetworkInfo, ProjectContext,
    RuntimeInfo, SshContext, SystemIdentity,
};
use crate::output::{RenderOptions, render_block, render_compact, render_config, render_json};

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn sample_info() -> MeInfo {
    MeInfo {
        identity: SystemIdentity {
            user: "tiger".into(),
            uid: 501,
            gid: 20,
            groups: vec![
                "staff".into(),
                "admin".into(),
                "_developer".into(),
                "access_bpf".into(),
                "everyone".into(),
            ],
            host: "MacBook.local".into(),
        },
        runtime: RuntimeInfo {
            shell: Some("zsh".into()),
            pid: 123,
            ppid: Some(1),
            tty: Some("ttys001".into()),
        },
        privilege: "user".into(),
        sudo: false,
        ssh: false,
        network: NetworkInfo {
            local_ips: vec![
                "192.168.1.10".into(),
                "10.0.0.5".into(),
                "172.16.0.3".into(),
            ],
        },
        context: ContextInfo {
            ssh: Some(SshContext {
                remote: true,
                connection: Some("client".into()),
            }),
            container: Some(ContainerContext {
                kind: "docker".into(),
                id: Some("abcdef123456".into()),
            }),
            project: Some(ProjectContext {
                kind: "rust".into(),
                version: Some("1.0".into()),
            }),
            git: Some(GitContext {
                branch: "main".into(),
            }),
        },
    }
}

#[test]
fn json_honors_selected_fields() {
    let rendered = render_json(&sample_info(), &[Field::User, Field::Host]).unwrap();
    assert!(rendered.contains("\"user\""));
    assert!(rendered.contains("\"host\""));
    assert!(!rendered.contains("\"uid\""));
}

#[test]
fn global_default_fields_still_include_shell() {
    assert!(Field::defaults().contains(&Field::Shell));
}

#[test]
fn config_default_output_still_includes_shell() {
    let rendered = render_config(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("shell = zsh"));
}

#[test]
fn json_default_output_still_includes_shell() {
    let rendered = render_json(&sample_info(), &Field::defaults()).unwrap();
    assert!(rendered.contains("\"shell\""));
}

#[test]
fn json_ends_with_newline_for_terminal_output() {
    let rendered = render_json(&sample_info(), &[Field::User]).unwrap();
    assert!(rendered.ends_with('\n'));
}

#[test]
fn config_format_is_stable() {
    let options = RenderOptions::plain_for_tests();
    let rendered = render_config(&sample_info(), &[Field::User, Field::Shell], &options);
    assert!(rendered.contains("user = tiger"));
    assert!(rendered.contains("shell = zsh"));
}

#[test]
fn block_truncates_groups_and_network_by_default() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("groups:     staff, admin, _developer (+2)"));
    assert!(rendered.contains("network:    192.168.1.10 (+2)"));
    assert!(!rendered.contains("access_bpf"));
    assert!(!rendered.contains("10.0.0.5"));
}

#[test]
fn block_header_uses_subtle_shell_separator() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.starts_with("tiger@MacBook  zsh\n\n"));
}

#[test]
fn block_default_output_suppresses_shell_body_row_only() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.starts_with("tiger@MacBook  zsh\n\n"));
    assert!(!rendered.contains("\nshell:"));
}

#[test]
fn block_uses_soft_section_spacing_without_titles() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("groups:     staff, admin, _developer (+2)\npid:"));
    assert!(rendered.contains("tty:        ttys001\nprivilege:"));
    assert!(!rendered.contains("runtime"));
    assert!(!rendered.contains("state"));
}

#[test]
fn colored_block_does_not_force_truecolor_on_values() {
    let mut options = RenderOptions::plain_for_tests();
    options.color = true;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);
    assert!(!rendered.contains("[38;2;"));
}

#[test]
fn colored_block_respects_light_theme_emphasis() {
    let mut dark = RenderOptions::plain_for_tests();
    dark.color = true;

    let mut light = RenderOptions::plain_for_tests();
    light.color = true;
    light.light_theme = true;

    let dark_rendered = render_block(&sample_info(), &Field::defaults(), &dark);
    let light_rendered = render_block(&sample_info(), &Field::defaults(), &light);

    assert_ne!(dark_rendered, light_rendered);
}

#[test]
fn icon_mode_on_keeps_block_output_plain_text_safe() {
    let mut options = RenderOptions::plain_for_tests();
    options.icons = IconMode::On;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);
    assert!(!rendered.contains('\u{f489}'));
    assert!(!rendered.contains('\u{f023}'));
    assert!(!rendered.contains('\u{f6ff}'));
}

#[test]
fn context_uses_soft_label_not_decorative_rule() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("\ncontext:    docker, rust (1.0) · git(main)\n"));
    assert!(!rendered.contains("--- context ---"));
    assert!(!rendered.contains("\ncontext\n\n"));
}

#[test]
fn block_output_renders_ssh_only_once() {
    let mut info = sample_info();
    info.ssh = true;
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert_eq!(rendered.matches("ssh:").count(), 1);
    assert!(rendered.contains("ssh:        yes"));
    assert!(!rendered.contains("ssh:       remote"));
}

#[test]
fn block_full_expands_groups_and_network() {
    let mut options = RenderOptions::plain_for_tests();
    options.full = true;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);
    assert!(rendered.contains("groups:\n  staff\n  admin\n  _developer\n  access_bpf\n  everyone"));
    assert!(rendered.contains("network:\n  192.168.1.10\n  10.0.0.5\n  172.16.0.3"));
}

#[test]
fn block_selector_does_not_render_context_section() {
    let rendered = render_block(
        &sample_info(),
        &[Field::Network],
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("network:"));
    assert!(!rendered.contains("context"));
    assert!(!rendered.contains("project:"));
}

#[test]
fn json_omits_null_context_members() {
    let mut info = sample_info();
    info.runtime.tty = None;
    info.context.ssh = None;
    info.context.container = None;
    let rendered = render_json(&info, &Field::defaults()).unwrap();
    assert!(!rendered.contains("null"));
    assert!(!rendered.contains("\"tty\""));
    assert!(!rendered.contains("\"container\""));
    assert!(rendered.contains("\"project\""));
    assert!(rendered.contains("\"git\""));
}

#[test]
fn json_project_includes_version_when_present() {
    let rendered = render_json(&sample_info(), &[Field::Context]).unwrap();
    assert!(rendered.contains("\"project\""));
    assert!(rendered.contains("\"version\": \"1.0\""));
    assert!(rendered.contains("\"git\""));
}

#[test]
fn json_project_omits_version_when_absent() {
    let mut info = sample_info();
    info.context.project = Some(ProjectContext {
        kind: "rust".into(),
        version: None,
    });
    let rendered = render_json(&info, &[Field::Context]).unwrap();
    assert!(rendered.contains("\"project\""));
    assert!(rendered.contains("\"kind\": \"rust\""));
    assert!(rendered.contains("\"git\""));
    assert!(!rendered.contains("\"version\""));
    assert!(!rendered.contains("null"));
}

#[test]
fn compact_limits_context_tags() {
    let mut info = sample_info();
    info.ssh = true;
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(" · ssh · rust git:main"));
    assert!(!rendered.contains("docker"));
    assert!(!rendered.contains("zsh"));
}

#[test]
fn compact_uses_detected_container_kind() {
    let mut info = sample_info();
    info.context.container = Some(ContainerContext {
        kind: "container".into(),
        id: None,
    });
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(" · container · "));
    assert!(!rendered.contains(" · docker · "));
}

#[test]
fn block_output_matches_golden_snapshot() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert_eq!(
        normalize_newlines(&rendered),
        normalize_newlines(include_str!("../../tests/golden/block.txt"))
    );
}

#[test]
fn compact_output_matches_golden_snapshot() {
    let rendered = render_compact(&sample_info(), &Field::defaults());
    assert_eq!(
        normalize_newlines(&rendered),
        normalize_newlines(include_str!("../../tests/golden/compact.txt"))
    );
}

#[test]
fn json_output_matches_golden_snapshot() {
    let rendered = render_json(
        &sample_info(),
        &[Field::User, Field::Host, Field::Sudo, Field::Ssh],
    )
    .unwrap();
    assert_eq!(
        normalize_newlines(&rendered),
        normalize_newlines(include_str!("../../tests/golden/json.txt"))
    );
}

#[test]
fn block_shows_both_project_and_git() {
    let info = sample_info();
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("rust (1.0) · git(main)"));
}

#[test]
fn block_shows_project_only_when_no_git() {
    let mut info = sample_info();
    info.context.git = None;
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("rust (1.0)"));
    assert!(!rendered.contains("git("));
}

#[test]
fn block_shows_git_only_when_no_project() {
    let mut info = sample_info();
    info.context.project = None;
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(!rendered.contains("rust"));
    assert!(rendered.contains("git(main)"));
}

#[test]
fn compact_shows_both_project_and_git() {
    let info = sample_info();
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains("rust git:main"));
}

#[test]
fn compact_shows_project_only_when_no_git() {
    let mut info = sample_info();
    info.context.git = None;
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(" · rust\n"));
    assert!(!rendered.contains("git:"));
}

#[test]
fn compact_shows_git_only_when_no_project() {
    let mut info = sample_info();
    info.context.project = None;
    let rendered = render_compact(&info, &Field::defaults());
    assert!(!rendered.contains("rust"));
    assert!(rendered.contains(" · git:main\n"));
}

#[test]
fn json_shows_both_project_and_git() {
    let info = sample_info();
    let rendered = render_json(&info, &[Field::Context]).unwrap();
    assert!(rendered.contains("\"project\""));
    assert!(rendered.contains("\"git\""));
}
