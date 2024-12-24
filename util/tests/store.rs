use util::tests::*;

#[tokio::test]
async fn get_test_state() {
    TestApiState::from_test_env().await.unwrap();
}
