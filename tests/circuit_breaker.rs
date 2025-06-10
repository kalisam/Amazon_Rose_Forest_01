use amazon_rose_forest::{CircuitBreaker, CircuitState};
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_transitions() {
    let cb = CircuitBreaker::new(
        "test",
        1,
        Duration::from_millis(50),
        Duration::from_millis(5),
    );
    assert_eq!(cb.get_state(), CircuitState::Closed);

    cb.on_failure().await;
    assert_eq!(cb.get_state(), CircuitState::Open);

    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cb.can_execute().await);
    assert_eq!(cb.get_state(), CircuitState::HalfOpen);

    cb.on_success().await;
    assert_eq!(cb.get_state(), CircuitState::Closed);
}
