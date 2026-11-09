use super::findings;
use kernel::config::{Ban, Config, File, Grant};

fn policy() -> Config {
    Config {
        file: File::default(),
    }
}

fn has(text: &str, law: &str, config: &Config) -> bool {
    findings("apps/web/src/view.svelte", text, config)
        .iter()
        .any(|finding| finding.law == law)
}

#[test]
fn inline() {
    let mut config = policy();
    config.file.ban.push(Ban {
        syntax: "style".into(),
        paths: vec!["apps/**".into()],
    });
    for text in [
        "<p style=\"color: red\">Lab</p>",
        "<p style={'color: red'}>Lab</p>",
        "<p style:color={'red'}>Lab</p>",
        "<p style:background-color={'red'}>Lab</p>",
        "<p style:--ink={'red'}>Lab</p>",
        "<p style:color|important={'red'}>Lab</p>",
        "<p style:color>Lab</p>",
        "<p>Lab</p><style lang=\"less\">p { color: red; }</style>",
    ] {
        assert!(has(text, "ban", &config), "{text}");
        assert!(!has(text, "coverage", &config), "{text}");
    }
}

#[test]
fn literal() {
    let mut config = policy();
    config.file.ban.push(Ban {
        syntax: "style".into(),
        paths: vec!["apps/**".into()],
    });
    for text in [
        "<p title=\"style:color=red\">style=red</p>",
        "<p style-code=\"red\">Lab</p>",
        "<p data-style=\"red\">Lab</p>",
        "<script>const style = 'red';</script><p>{style}</p>",
        "<!-- <p style=\"color:red\"> --> <p>Lab</p>",
    ] {
        assert!(!has(text, "ban", &config), "{text}");
        assert!(!has(text, "coverage", &config), "{text}");
    }
}

#[test]
fn granted() {
    let mut config = policy();
    config.file.grant.push(Grant {
        syntax: "style".into(),
        paths: vec!["library/**".into()],
    });
    for text in [
        "<p style=\"color:red\">Lab</p>",
        "<p style:color={'red'}>Lab</p>",
    ] {
        assert!(has(text, "grant", &config), "{text}");
        assert!(
            !findings("library/view.svelte", text, &config)
                .iter()
                .any(|finding| finding.law == "grant"),
            "{text}"
        );
    }
    config.file.ban.push(Ban {
        syntax: "style".into(),
        paths: vec!["library/**".into()],
    });
    assert!(
        findings(
            "library/view.svelte",
            "<p style:color={'red'}>Lab</p>",
            &config
        )
        .iter()
        .any(|finding| finding.law == "ban")
    );
}

#[test]
fn languages() {
    for lang in ["css", "scss", "less"] {
        for value in [lang.to_string(), format!("\"{lang}\""), format!("'{lang}'")] {
            let text = format!("<style lang={value}>p {{ color: red; }}</style>");
            assert!(!has(&text, "coverage", &policy()), "{text}");
        }
    }
    assert!(!has(
        "<style>p { color: red; }</style>",
        "coverage",
        &policy()
    ));
    for text in [
        "<style lang=\"stylus\">p\n  color red\n</style>",
        "<style lang='sass'>p\n  color: red\n</style>",
        "<style lang={language}>p { color: red; }</style>",
        "<style lang=stylus>p\n  color red\n</style>",
        "<script lang=\"coffee\">square = (x) -> x * x</script><p>Lab</p>",
    ] {
        assert!(has(text, "coverage", &policy()), "{text}");
    }
}
