use bsuite_core::RoutingKey;

#[test]
fn bground_routing_key_uses_the_canonical_core_entry() {
    let routing_key = bground::routing_key();

    assert_eq!(routing_key, RoutingKey::BGround);
    assert_eq!(routing_key, RoutingKey::bground());
    assert_eq!(routing_key.stable_name(), "bground");
}

#[test]
fn bground_routing_key_is_present_once_in_the_core_key_set() {
    let occurrences = RoutingKey::ALL
        .into_iter()
        .filter(|routing_key| *routing_key == bground::routing_key())
        .count();

    assert_eq!(occurrences, 1);
}
