use crate::config::IconMode;
use crate::model::{
    ContainerContext, ContextInfo, Field, MeInfo, NetworkInfo, ProjectContext, RuntimeInfo,
    SshContext, SystemIdentity,
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
                version: Some("rustc 1.0".into()),
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
fn block_uses_soft_section_spacing_without_titles() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("groups:     staff, admin, _developer (+2)\nshell:"));
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
    assert!(rendered.contains("\ncontext:    docker, rust (rustc 1.0)\n"));
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
}

#[test]
fn compact_limits_context_tags() {
    let mut info = sample_info();
    info.ssh = true;
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(" · ssh · rust"));
    assert!(!rendered.contains("docker"));
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
