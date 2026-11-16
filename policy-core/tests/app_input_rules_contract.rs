// SPDX-License-Identifier: MIT
use chewing_policy_core::app_input_rules::{AppInputRule,AppInputRulesTracker,RuleMode,normalize_canonical_app_id,parse_and_validate_rules};
const APP: &str = r"c:\apps\editor.exe";
fn rule() -> AppInputRule { AppInputRule {app_id:APP.into(),display_name:"Editor".into(),mode:RuleMode::English} }
#[test]
fn manual_action_does_not_enable_disabled_or_unconfigured_visit() {
    for (enabled,rules) in [(false,vec![rule()]),(true,vec![])] {
        let mut t=AppInputRulesTracker::new(enabled,&rules);
        let (g,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
        assert!(!t.manual(g,Some(APP),RuleMode::Bopomofo));
        assert_eq!(t.target(g,Some(APP),true,true,false),None);
    }
}
#[test]
fn stale_or_unknown_clients_cannot_override_current_visit() {
    let mut t=AppInputRulesTracker::new(true,&[rule()]);
    let (old,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
    t.on_foreground_app_changed(None);
    let (new,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
    assert_ne!(old,new);
    for (g,id) in [(old,Some(APP)),(new,None),(new,Some(r"c:\other.exe"))] {
        assert!(!t.manual(g,id,RuleMode::Bopomofo));
        assert_eq!(t.target(g,id,true,true,false),None);
    }
    assert_eq!(t.target(new,Some(APP),true,true,false),Some(RuleMode::English));
}
#[test]
fn defer_does_not_consume_target_and_manual_cancels_default() {
    let mut t=AppInputRulesTracker::new(true,&[rule()]);
    let(g,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
    for (safe,active,secure) in [(false,true,false),(true,false,false),(true,true,true)] {
        assert_eq!(t.target(g,Some(APP),safe,active,secure),None);
    }
    assert!(t.manual(g,Some(APP),RuleMode::Bopomofo));
    assert_eq!(t.target(g,Some(APP),true,true,false),Some(RuleMode::Bopomofo));
    assert_eq!(t.on_foreground_app_changed(Some(APP)),Some((g,Some(RuleMode::Bopomofo))));
}
#[test]
fn delete_and_disable_only_affect_next_visit_not_deferred_target() {
    let mut t=AppInputRulesTracker::new(true,&[rule()]);
    let(g,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
    t.update_config(false,&[]);
    assert_eq!(t.target(g,Some(APP),false,true,false),None);
    assert_eq!(t.target(g,Some(APP),true,true,false),Some(RuleMode::English));
    t.on_foreground_app_changed(Some(r"c:\another.exe"));
    let(new,_)=t.on_foreground_app_changed(Some(APP)).unwrap();
    assert_eq!(t.target(new,Some(APP),true,true,false),None);
}
#[test]
fn leading_spaces_in_real_path_components_are_not_an_alias() {
    assert_ne!(normalize_canonical_app_id(r"C:\ Apps\editor.exe"),normalize_canonical_app_id(r"C:\Apps\editor.exe"));
    assert_eq!(normalize_canonical_app_id(r"\\?\C:\Apps\Editor.exe"),Some(APP.into()));
}
#[test]
fn duplicate_import_keeps_first_mode_and_unicode_display_name() {
    let mut first=rule();first.display_name="工程".repeat(100);first.mode=RuleMode::Bopomofo;
    let document=serde_json::json!({"version":1,"rules":[first,rule()]});
    let rules=parse_and_validate_rules(&document.to_string());
    assert_eq!(rules.len(),1);assert_eq!(rules[0].mode,RuleMode::Bopomofo);
}
