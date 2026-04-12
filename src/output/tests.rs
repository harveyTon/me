use crate::config::IconMode;
use crate::model::{
    ContainerContext, ContextInfo, Field, GitContext, MeInfo, NetworkInfo, ProjectContext, PwdInfo,
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
            ipv4_local_ips: vec![
                "192.168.1.10".into(),
                "10.0.0.5".into(),
                "172.16.0.3".into(),
            ],
            ipv6_local_ips: vec!["fd12::1".into(), "fd12::2".into()],
        },
        pwd: Some(PwdInfo {
            raw: "/Users/tiger/dev/me".into(),
            display: "/Users/tiger/dev/me".into(),
        }),
        context: ContextInfo {
            ssh: Some(SshContext {
                remote: true,
                connection: Some("client".into()),
            }),
            container: Some(ContainerContext {
                kind: "docker".into(),
                id: Some("abcdef123456".into()),
            }),
            projects: vec![ProjectContext {
                kind: "rust".into(),
                version: Some("1.0".into()),
                project_name: None,
                service_count: None,
                details: Vec::new(),
            }],
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
fn json_default_output_includes_pwd_object() {
    let rendered = render_json(&sample_info(), &Field::defaults()).unwrap();
    assert!(rendered.contains("\"pwd\""));
    assert!(rendered.contains("\"raw\": \"/Users/tiger/dev/me\""));
    assert!(rendered.contains("\"display\": \"/Users/tiger/dev/me\""));
    assert!(rendered.contains("\"ipv4_local_ips\""));
    assert!(rendered.contains("\"ipv6_local_ips\""));
}

#[test]
fn json_ends_with_newline_for_terminal_output() {
    let rendered = render_json(&sample_info(), &[Field::User]).unwrap();
    assert!(rendered.ends_with('\n'));
}

#[test]
fn config_format_is_stable() {
    let options = RenderOptions::plain_for_tests();
    let rendered = render_config(
        &sample_info(),
        &[Field::User, Field::Shell, Field::Pwd],
        &options,
    );
    assert!(rendered.contains("user = tiger"));
    assert!(rendered.contains("shell = zsh"));
    assert!(rendered.contains("pwd = /Users/tiger/dev/me"));
}

#[test]
fn block_truncates_groups_and_network_by_default() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("  groups:"));
    assert!(rendered.contains("staff, admin, _developer (+2)"));
    assert!(rendered.contains("  ipv4:"));
    assert!(rendered.contains("192.168.1.10 (+2)"));
    assert!(rendered.contains("  ipv6:"));
    assert!(rendered.contains("fd12::1 (+1)"));
    assert!(!rendered.contains("summary:"));
}

#[test]
fn block_removes_header_and_moves_identity_into_body() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(!rendered.starts_with("tiger@MacBook"));
    assert!(rendered.starts_with("identity:\n"));
    assert!(rendered.contains("  user:"));
    assert!(rendered.contains("tiger\n"));
    assert!(rendered.contains("  host:"));
    assert!(rendered.contains("MacBook\n"));
    assert!(rendered.contains("  shell:"));
    assert!(rendered.contains("zsh\n"));
}

#[test]
fn block_groups_appear_in_fixed_order() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );

    let identity = rendered.find("identity:\n").unwrap();
    let system = rendered.find("\nsystem:\n").unwrap();
    let session = rendered.find("\nsession:\n").unwrap();
    let network = rendered.find("\nnetwork:\n").unwrap();
    let location = rendered.find("\nlocation:\n").unwrap();

    assert!(identity < system);
    assert!(system < session);
    assert!(session < network);
    assert!(network < location);
}

#[test]
fn block_default_output_places_pwd_and_context_in_location_group() {
    let rendered = render_block(
        &sample_info(),
        &Field::defaults(),
        &RenderOptions::plain_for_tests(),
    );
    assert!(rendered.contains("location:\n"));
    assert!(rendered.contains("  pwd:"));
    assert!(rendered.contains("/Users/tiger/dev/me\n"));
    assert!(rendered.contains("  context:"));
    assert!(rendered.contains("docker, rust 1.0 · git(main)\n"));
}

#[test]
fn colored_block_does_not_force_truecolor_on_values() {
    let mut options = RenderOptions::plain_for_tests();
    options.color = true;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);
    assert!(!rendered.contains("[38;2;"));
}

#[test]
fn colored_block_labels_stay_ansi_safe_in_light_and_dark_modes() {
    let mut light = RenderOptions::plain_for_tests();
    light.color = true;
    light.light_theme = true;

    let light_rendered = render_block(&sample_info(), &Field::defaults(), &light);
    let mut dark = RenderOptions::plain_for_tests();
    dark.color = true;
    let dark_rendered = render_block(&sample_info(), &Field::defaults(), &dark);

    assert!(dark_rendered.contains("\u{1b}["));
    assert!(light_rendered.contains("\u{1b}["));
    assert!(!dark_rendered.contains("[38;2;"));
    assert!(!light_rendered.contains("[38;2;"));
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
    assert!(rendered.contains("\n  context:"));
    assert!(rendered.contains("docker, rust 1.0 · git(main)\n"));
    assert!(!rendered.contains("--- context ---"));
}

#[test]
fn block_output_renders_ssh_only_once() {
    let mut info = sample_info();
    info.ssh = true;
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert_eq!(rendered.matches("ssh:").count(), 1);
    assert!(rendered.contains("  ssh:        yes"));
}

#[test]
fn block_full_expands_groups_and_network() {
    let mut options = RenderOptions::plain_for_tests();
    options.full = true;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);
    assert!(rendered.contains("identity:\n") && rendered.contains("\nsystem:\n"));
    assert!(
        rendered.contains(
            "  groups:\n    staff\n    admin\n    _developer\n    access_bpf\n    everyone"
        )
    );
    assert!(
        rendered.contains(
            "network:\n  ipv4:\n    192.168.1.10\n    10.0.0.5\n    172.16.0.3\n  ipv6:\n    fd12::1\n    fd12::2"
        )
    );
    assert!(!rendered.contains("summary:"));
}

#[test]
fn block_full_uses_same_group_structure() {
    let mut options = RenderOptions::plain_for_tests();
    options.full = true;
    let rendered = render_block(&sample_info(), &Field::defaults(), &options);

    let identity = rendered.find("identity:\n").unwrap();
    let system = rendered.find("\nsystem:\n").unwrap();
    let session = rendered.find("\nsession:\n").unwrap();
    let network = rendered.find("\nnetwork:\n").unwrap();
    let location = rendered.find("\nlocation:\n").unwrap();

    assert!(identity < system);
    assert!(system < session);
    assert!(session < network);
    assert!(network < location);
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
fn block_omits_context_cleanly_when_absent() {
    let mut info = sample_info();
    info.context = ContextInfo::default();
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("location:\n"));
    assert!(rendered.contains("  pwd:"));
    assert!(rendered.contains("/Users/tiger/dev/me\n"));
    assert!(!rendered.contains("context:"));
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
    assert!(rendered.contains("\"projects\""));
    assert!(rendered.contains("\"git\""));
}

#[test]
fn json_projects_include_version_when_present() {
    let rendered = render_json(&sample_info(), &[Field::Context]).unwrap();
    assert!(rendered.contains("\"projects\""));
    assert!(rendered.contains("\"version\": \"1.0\""));
    assert!(rendered.contains("\"git\""));
}

#[test]
fn json_project_omits_version_when_absent() {
    let mut info = sample_info();
    info.context.projects = vec![ProjectContext {
        kind: "rust".into(),
        version: None,
        project_name: None,
        service_count: None,
        details: Vec::new(),
    }];
    let rendered = render_json(&info, &[Field::Context]).unwrap();
    assert!(rendered.contains("\"projects\""));
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
    assert!(rendered.contains(" · ssh · rust 1.0 · git:main · me"));
    assert!(!rendered.contains("docker"));
}

#[test]
fn compact_uses_detected_container_kind() {
    let mut info = sample_info();
    info.context.container = Some(ContainerContext {
        kind: "container".into(),
        id: None,
    });
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(" · container · rust 1.0 · git:main · me"));
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
        &[
            Field::User,
            Field::Host,
            Field::Pwd,
            Field::Sudo,
            Field::Ssh,
        ],
    )
    .unwrap();
    assert_eq!(
        normalize_newlines(&rendered),
        normalize_newlines(include_str!("../../tests/golden/json.txt"))
    );
}

#[test]
fn block_shows_both_projects_and_git() {
    let mut info = sample_info();
    info.context.projects.push(ProjectContext {
        kind: "python".into(),
        version: Some("3.12".into()),
        project_name: None,
        service_count: None,
        details: vec![".venv".into()],
    });
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("context:"));
    assert!(rendered.contains("docker, rust 1.0 · python 3.12 (.venv) · git(main)"));
}

#[test]
fn block_folds_project_related_items_after_three_entries() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
        ProjectContext {
            kind: "go".into(),
            version: Some("1.24.0".into()),
            project_name: None,
            service_count: None,
            details: Vec::new(),
        },
        ProjectContext {
            kind: "java".into(),
            version: Some("21".into()),
            project_name: None,
            service_count: None,
            details: vec!["gradle".into()],
        },
    ];
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(
        rendered
            .contains("docker, node 22.0.0 (pnpm, turbo) · python 3.12 (.venv) · go 1.24.0 (+2)")
    );
    assert!(!rendered.contains("java 21 (gradle)"));
}

#[test]
fn block_shows_exactly_three_context_items_without_folding() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
    ];
    info.context.git = Some(GitContext {
        branch: "feature/nextjs-frontend".into(),
    });

    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());

    assert!(rendered.contains(
        "docker, node 22.0.0 (pnpm, turbo) · python 3.12 (.venv) · git(feature/nextjs-frontend)"
    ));
    assert!(!rendered.contains("git(feature/nextjs-frontend) (+"));
}

#[test]
fn compact_shows_multiple_project_related_items_in_priority_order() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into()],
        },
    ];
    let rendered = render_compact(&info, &Field::defaults());
    assert!(rendered.contains(
        "tiger@MacBook · docker · node 22.0.0 (pnpm, turbo) · python 3.12 (.venv) · git:main · me\n"
    ));
}

#[test]
fn compact_folds_context_items_after_three_entries() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into(), "nx".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
        ProjectContext {
            kind: "go".into(),
            version: Some("1.24.0".into()),
            project_name: None,
            service_count: None,
            details: Vec::new(),
        },
        ProjectContext {
            kind: "java".into(),
            version: Some("21".into()),
            project_name: None,
            service_count: None,
            details: vec!["gradle".into()],
        },
    ];
    info.context.git = Some(GitContext {
        branch: "feature/nextjs-frontend".into(),
    });

    let rendered = render_compact(&info, &Field::defaults());

    assert!(rendered.contains(
        "tiger@MacBook · docker · node 22.0.0 (pnpm, turbo) · python 3.12 (.venv) · go 1.24.0 (+2) · me\n"
    ));
    assert!(!rendered.contains("java 21 (gradle)"));
    assert!(!rendered.contains("git:feature/nextjs-frontend"));
}

#[test]
fn compact_shows_exactly_three_context_items_without_folding() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into(), "nx".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
    ];
    info.context.git = Some(GitContext {
        branch: "feature/nextjs-frontend".into(),
    });

    let rendered = render_compact(&info, &Field::defaults());

    assert!(rendered.contains(
        "tiger@MacBook · docker · node 22.0.0 (pnpm, turbo) · python 3.12 (.venv) · git:feature/nextjs-frontend · me\n"
    ));
    assert!(!rendered.contains("git:feature/nextjs-frontend (+"));
}

#[test]
fn node_version_display_is_plain_across_outputs() {
    let mut info = sample_info();
    info.context.projects = vec![ProjectContext {
        kind: "node".into(),
        version: Some("20.19.6".into()),
        project_name: None,
        service_count: None,
        details: vec!["pnpm".into(), "turbo".into()],
    }];
    info.context.git = Some(GitContext {
        branch: "v2.2.4".into(),
    });

    let block = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    let compact = render_compact(&info, &Field::defaults());
    let config = render_config(&info, &[Field::Context], &RenderOptions::plain_for_tests());
    let json = render_json(&info, &[Field::Context]).unwrap();

    assert!(block.contains("context:"));
    assert!(block.contains("docker, node 20.19.6 (pnpm, turbo) · git(v2.2.4)"));
    assert!(compact.contains("node 20.19.6 (pnpm, turbo)"));
    assert!(
        config.contains("context = docker:abcdef123456, node 20.19.6 (pnpm, turbo), git:v2.2.4")
    );
    assert!(json.contains("\"version\": \"20.19.6\""));
    assert!(json.contains("\"details\": ["));
}

#[test]
fn node_text_output_limits_enhancements_to_two_items() {
    let mut info = sample_info();
    info.context.projects = vec![ProjectContext {
        kind: "node".into(),
        version: Some("24.14.1".into()),
        project_name: None,
        service_count: None,
        details: vec![
            "pnpm".into(),
            "turbo".into(),
            "nx".into(),
            "workspace".into(),
        ],
    }];

    let block = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    let compact = render_compact(&info, &Field::defaults());

    assert!(block.contains("node 24.14.1 (pnpm, turbo)"));
    assert!(!block.contains("nx"));
    assert!(!block.contains("workspace"));
    assert!(compact.contains("node 24.14.1 (pnpm, turbo)"));
    assert!(!compact.contains("nx"));
    assert!(!compact.contains("workspace"));
}

#[test]
fn git_branch_is_not_truncated_in_text_output() {
    let mut info = sample_info();
    info.context.projects = vec![ProjectContext {
        kind: "python".into(),
        version: Some("3.12".into()),
        project_name: None,
        service_count: None,
        details: vec![".venv".into()],
    }];
    info.context.git = Some(GitContext {
        branch: "feature/nextjs-frontend".into(),
    });

    let block = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    let compact = render_compact(&info, &Field::defaults());

    assert!(block.contains("git(feature/nextjs-frontend)"));
    assert!(compact.contains("git:feature/nextjs-frontend"));
}

#[test]
fn shared_text_context_semantics_stay_consistent() {
    let mut info = sample_info();
    info.context.container = Some(ContainerContext {
        kind: "container".into(),
        id: Some("abcdef123456".into()),
    });
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("20.19.6".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
    ];
    info.context.git = Some(GitContext {
        branch: "feature/login".into(),
    });

    let block = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    let compact = render_compact(&info, &Field::defaults());
    let config = render_config(&info, &[Field::Context], &RenderOptions::plain_for_tests());

    assert!(block.contains("context:"));
    assert!(
        block.contains("container, node 20.19.6 (pnpm) · python 3.12 (.venv) · git(feature/login)")
    );
    assert!(compact.contains(" · container · "));
    assert!(compact.contains("git:feature/login"));
    assert!(config.contains("context = container:abcdef123456, node 20.19.6 (pnpm), python 3.12 (.venv), git:feature/login"));
}

#[test]
fn json_shows_projects_and_git() {
    let info = sample_info();
    let rendered = render_json(&info, &[Field::Context]).unwrap();
    assert!(rendered.contains("\"projects\""));
    assert!(rendered.contains("\"git\""));
}

#[test]
fn network_output_remains_unchanged_with_dense_context() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("22.0.0".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into(), "turbo".into(), "nx".into()],
        },
        ProjectContext {
            kind: "python".into(),
            version: Some("3.12".into()),
            project_name: None,
            service_count: None,
            details: vec![".venv".into()],
        },
        ProjectContext {
            kind: "go".into(),
            version: Some("1.24.0".into()),
            project_name: None,
            service_count: None,
            details: Vec::new(),
        },
        ProjectContext {
            kind: "java".into(),
            version: Some("21".into()),
            project_name: None,
            service_count: None,
            details: vec!["gradle".into()],
        },
    ];
    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("network:\n"));
    assert!(rendered.contains("  ipv4:"));
    assert!(rendered.contains("192.168.1.10 (+2)"));
    assert!(rendered.contains("  ipv6:"));
    assert!(rendered.contains("fd12::1 (+1)"));
    assert!(!rendered.contains("summary:"));
}

#[test]
fn block_network_omits_missing_address_family_cleanly() {
    let mut info = sample_info();
    info.network.ipv6_local_ips.clear();

    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("network:\n  ipv4:"));
    assert!(rendered.contains("192.168.1.10 (+2)"));
    assert!(!rendered.contains("\n  ipv6:"));

    info.network.ipv4_local_ips.clear();
    info.network.ipv6_local_ips = vec!["fd12::1".into(), "fd12::2".into()];

    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());
    assert!(rendered.contains("network:\n  ipv6:"));
    assert!(rendered.contains("fd12::1 (+1)"));
    assert!(!rendered.contains("\n  ipv4:"));
}

#[test]
fn block_output_formats_docker_compose_project_summary() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("24.14.1".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into()],
        },
        ProjectContext {
            kind: "docker compose".into(),
            version: None,
            project_name: Some("myapp".into()),
            service_count: Some(6),
            details: Vec::new(),
        },
    ];

    let rendered = render_block(&info, &Field::defaults(), &RenderOptions::plain_for_tests());

    assert!(
        rendered.contains(
            "docker, node 24.14.1 (pnpm) · docker compose (myapp, 6 services) · git(main)"
        )
    );
}

#[test]
fn compact_output_formats_docker_compose_project_summary() {
    let mut info = sample_info();
    info.context.projects = vec![
        ProjectContext {
            kind: "node".into(),
            version: Some("24.14.1".into()),
            project_name: None,
            service_count: None,
            details: vec!["pnpm".into()],
        },
        ProjectContext {
            kind: "docker compose".into(),
            version: None,
            project_name: Some("myapp".into()),
            service_count: Some(6),
            details: Vec::new(),
        },
    ];

    let rendered = render_compact(&info, &Field::defaults());

    assert!(rendered.contains(
        "tiger@MacBook · docker · node 24.14.1 (pnpm) · docker-compose:myapp · git:main · me\n"
    ));
}

#[test]
fn json_output_preserves_docker_compose_project_fields() {
    let mut info = sample_info();
    info.context.projects = vec![ProjectContext {
        kind: "docker compose".into(),
        version: None,
        project_name: Some("myapp".into()),
        service_count: Some(6),
        details: Vec::new(),
    }];

    let rendered = render_json(&info, &[Field::Context]).unwrap();

    assert!(rendered.contains("\"kind\": \"docker compose\""));
    assert!(rendered.contains("\"project_name\": \"myapp\""));
    assert!(rendered.contains("\"service_count\": 6"));
}
