use investwise_core::profiles::profiles_model::ProfileSharingRule;

#[test]
fn profile_sharing_rule_defaults_to_private() {
    assert_eq!(ProfileSharingRule::from_str("unknown"), ProfileSharingRule::Private);
    assert_eq!(ProfileSharingRule::Private.as_str(), "private");
}

#[test]
fn profile_sharing_rule_supports_family_modes() {
    assert_eq!(ProfileSharingRule::from_str("family_read_only"), ProfileSharingRule::FamilyReadOnly);
    assert_eq!(ProfileSharingRule::from_str("family_read_write"), ProfileSharingRule::FamilyReadWrite);
}
