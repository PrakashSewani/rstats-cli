use rstats::history::History;

#[test]
fn history_is_bounded_and_resettable() {
    let mut history = History::new(2);
    history.push(1.0);
    history.push(2.0);
    history.push(3.0);
    assert_eq!(history.as_vec(), vec![2.0, 3.0]);
    history.clear();
    assert!(history.is_empty());
}
