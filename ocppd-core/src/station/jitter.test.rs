use super::*;

#[test]
fn given_same_station_and_attempt_when_drawn_twice_then_same_fraction() {
    let first = fraction("CP1", 0);

    let second = fraction("CP1", 0);

    assert_eq!(first, second);
}

#[test]
fn given_another_attempt_or_station_when_drawn_then_another_fraction() {
    let base = fraction("CP1", 0);

    let others = [fraction("CP1", 1), fraction("CP2", 0)];

    assert_ne!(others[0], base);
    assert_ne!(others[1], base);
}

#[test]
fn given_fleet_of_100_when_drawn_then_every_decile_of_0_to_1_used() {
    let names: Vec<String> = (1..=100).map(|n| format!("CP{n}")).collect();

    let values: Vec<f64> = names.iter().map(|name| fraction(name, 0)).collect();

    let mut deciles = [0; 10];
    for value in values {
        assert!((0.0..1.0).contains(&value));
        deciles[(value * 10.0) as usize] += 1;
    }
    assert!(deciles.iter().all(|&count| count > 0), "{deciles:?}");
}
