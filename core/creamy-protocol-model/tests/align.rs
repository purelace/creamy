use creamy_protocol_model::Align;

#[test]
fn validate() {
    assert_eq!(Align::B1.discriminant(), 0);
    assert_eq!(Align::B2.discriminant(), 1);
    assert_eq!(Align::B4.discriminant(), 2);
    assert_eq!(Align::B8.discriminant(), 3);
    assert_eq!(Align::B16.discriminant(), 4);

    assert_eq!(Align::B1.size(), 1);
    assert_eq!(Align::B2.size(), 2);
    assert_eq!(Align::B4.size(), 4);
    assert_eq!(Align::B8.size(), 8);
    assert_eq!(Align::B16.size(), 16);

    assert_eq!(Align::discriminant_from_size(1), 0);
    assert_eq!(Align::discriminant_from_size(2), 1);
    assert_eq!(Align::discriminant_from_size(4), 2);
    assert_eq!(Align::discriminant_from_size(8), 3);
    assert_eq!(Align::discriminant_from_size(16), 4);
}
