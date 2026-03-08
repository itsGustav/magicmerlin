use std::time::Duration;

use magicmerlin_gateway::run_queue::{RunQueue, RunQueueConfig, RunStatus};
use magicmerlin_gateway::ws::{parse_bearer_auth, reconnect_backoff, should_retry_connection};

#[tokio::test]
async fn matrix_smoke_queue_lifecycle() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 5,
        default_timeout: Duration::from_secs(30),
    });

    queue.enqueue("base", "run-1", Some(Duration::from_secs(10))).await.unwrap();
    queue.wait_turn("base", "run-1", Duration::from_secs(1)).await.unwrap();
    queue.complete("base", "run-1", RunStatus::Completed, None).await;

    let runs = queue.list_session_runs("base").await;
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, RunStatus::Completed);
}

#[test]
fn matrix_smoke_auth_parse() {
    assert_eq!(parse_bearer_auth("Bearer token"), Some("token"));
    assert_eq!(parse_bearer_auth("bearer token"), Some("token"));
    assert_eq!(parse_bearer_auth("Token token"), None);
}

#[test]
fn matrix_smoke_retry_policy() {
    assert!(should_retry_connection("timeout"));
    assert!(should_retry_connection("connection reset"));
    assert!(!should_retry_connection("401 unauthorized"));
}

#[test]
fn matrix_smoke_backoff() {
    assert_eq!(reconnect_backoff(0), Duration::from_secs(1));
    assert_eq!(reconnect_backoff(2), Duration::from_secs(4));
}


#[test]
fn retry_policy_case_1() {
    let transient = [
        "timeout #1",
        "connection reset by peer #1",
        "broken pipe #1",
        "temporarily unavailable #1",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #1",
        "permission denied #1",
        "bad request #1",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_1() {
    let good = format!("Bearer tok_1");
    let lower = format!("bearer tok_1");
    let bad = format!("Basic tok_1");

    assert_eq!(parse_bearer_auth(&good), Some("tok_1"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_1"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_1() {
    let expected = if 1 < 9 { 2u64.pow(1u32) } else { 256 };
    assert_eq!(reconnect_backoff(1), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_2() {
    let transient = [
        "timeout #2",
        "connection reset by peer #2",
        "broken pipe #2",
        "temporarily unavailable #2",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #2",
        "permission denied #2",
        "bad request #2",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_2() {
    let good = format!("Bearer tok_2");
    let lower = format!("bearer tok_2");
    let bad = format!("Basic tok_2");

    assert_eq!(parse_bearer_auth(&good), Some("tok_2"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_2"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_2() {
    let expected = if 2 < 9 { 2u64.pow(2u32) } else { 256 };
    assert_eq!(reconnect_backoff(2), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_3() {
    let transient = [
        "timeout #3",
        "connection reset by peer #3",
        "broken pipe #3",
        "temporarily unavailable #3",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #3",
        "permission denied #3",
        "bad request #3",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_3() {
    let good = format!("Bearer tok_3");
    let lower = format!("bearer tok_3");
    let bad = format!("Basic tok_3");

    assert_eq!(parse_bearer_auth(&good), Some("tok_3"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_3"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_3() {
    let expected = if 3 < 9 { 2u64.pow(3u32) } else { 256 };
    assert_eq!(reconnect_backoff(3), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_4() {
    let transient = [
        "timeout #4",
        "connection reset by peer #4",
        "broken pipe #4",
        "temporarily unavailable #4",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #4",
        "permission denied #4",
        "bad request #4",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_4() {
    let good = format!("Bearer tok_4");
    let lower = format!("bearer tok_4");
    let bad = format!("Basic tok_4");

    assert_eq!(parse_bearer_auth(&good), Some("tok_4"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_4"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_4() {
    let expected = if 4 < 9 { 2u64.pow(4u32) } else { 256 };
    assert_eq!(reconnect_backoff(4), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_5() {
    let transient = [
        "timeout #5",
        "connection reset by peer #5",
        "broken pipe #5",
        "temporarily unavailable #5",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #5",
        "permission denied #5",
        "bad request #5",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_5() {
    let good = format!("Bearer tok_5");
    let lower = format!("bearer tok_5");
    let bad = format!("Basic tok_5");

    assert_eq!(parse_bearer_auth(&good), Some("tok_5"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_5"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_5() {
    let expected = if 5 < 9 { 2u64.pow(5u32) } else { 256 };
    assert_eq!(reconnect_backoff(5), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_6() {
    let transient = [
        "timeout #6",
        "connection reset by peer #6",
        "broken pipe #6",
        "temporarily unavailable #6",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #6",
        "permission denied #6",
        "bad request #6",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_6() {
    let good = format!("Bearer tok_6");
    let lower = format!("bearer tok_6");
    let bad = format!("Basic tok_6");

    assert_eq!(parse_bearer_auth(&good), Some("tok_6"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_6"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_6() {
    let expected = if 6 < 9 { 2u64.pow(6u32) } else { 256 };
    assert_eq!(reconnect_backoff(6), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_7() {
    let transient = [
        "timeout #7",
        "connection reset by peer #7",
        "broken pipe #7",
        "temporarily unavailable #7",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #7",
        "permission denied #7",
        "bad request #7",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_7() {
    let good = format!("Bearer tok_7");
    let lower = format!("bearer tok_7");
    let bad = format!("Basic tok_7");

    assert_eq!(parse_bearer_auth(&good), Some("tok_7"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_7"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_7() {
    let expected = if 7 < 9 { 2u64.pow(7u32) } else { 256 };
    assert_eq!(reconnect_backoff(7), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_8() {
    let transient = [
        "timeout #8",
        "connection reset by peer #8",
        "broken pipe #8",
        "temporarily unavailable #8",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #8",
        "permission denied #8",
        "bad request #8",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_8() {
    let good = format!("Bearer tok_8");
    let lower = format!("bearer tok_8");
    let bad = format!("Basic tok_8");

    assert_eq!(parse_bearer_auth(&good), Some("tok_8"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_8"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_8() {
    let expected = if 8 < 9 { 2u64.pow(8u32) } else { 256 };
    assert_eq!(reconnect_backoff(8), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_9() {
    let transient = [
        "timeout #9",
        "connection reset by peer #9",
        "broken pipe #9",
        "temporarily unavailable #9",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #9",
        "permission denied #9",
        "bad request #9",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_9() {
    let good = format!("Bearer tok_9");
    let lower = format!("bearer tok_9");
    let bad = format!("Basic tok_9");

    assert_eq!(parse_bearer_auth(&good), Some("tok_9"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_9"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_9() {
    let expected = if 9 < 9 { 2u64.pow(9u32) } else { 256 };
    assert_eq!(reconnect_backoff(9), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_10() {
    let transient = [
        "timeout #10",
        "connection reset by peer #10",
        "broken pipe #10",
        "temporarily unavailable #10",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #10",
        "permission denied #10",
        "bad request #10",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_10() {
    let good = format!("Bearer tok_10");
    let lower = format!("bearer tok_10");
    let bad = format!("Basic tok_10");

    assert_eq!(parse_bearer_auth(&good), Some("tok_10"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_10"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_10() {
    let expected = if 10 < 9 { 2u64.pow(10u32) } else { 256 };
    assert_eq!(reconnect_backoff(10), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_11() {
    let transient = [
        "timeout #11",
        "connection reset by peer #11",
        "broken pipe #11",
        "temporarily unavailable #11",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #11",
        "permission denied #11",
        "bad request #11",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_11() {
    let good = format!("Bearer tok_11");
    let lower = format!("bearer tok_11");
    let bad = format!("Basic tok_11");

    assert_eq!(parse_bearer_auth(&good), Some("tok_11"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_11"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_11() {
    let expected = if 11 < 9 { 2u64.pow(11u32) } else { 256 };
    assert_eq!(reconnect_backoff(11), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_12() {
    let transient = [
        "timeout #12",
        "connection reset by peer #12",
        "broken pipe #12",
        "temporarily unavailable #12",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #12",
        "permission denied #12",
        "bad request #12",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_12() {
    let good = format!("Bearer tok_12");
    let lower = format!("bearer tok_12");
    let bad = format!("Basic tok_12");

    assert_eq!(parse_bearer_auth(&good), Some("tok_12"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_12"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_12() {
    let expected = if 12 < 9 { 2u64.pow(12u32) } else { 256 };
    assert_eq!(reconnect_backoff(12), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_13() {
    let transient = [
        "timeout #13",
        "connection reset by peer #13",
        "broken pipe #13",
        "temporarily unavailable #13",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #13",
        "permission denied #13",
        "bad request #13",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_13() {
    let good = format!("Bearer tok_13");
    let lower = format!("bearer tok_13");
    let bad = format!("Basic tok_13");

    assert_eq!(parse_bearer_auth(&good), Some("tok_13"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_13"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_13() {
    let expected = if 13 < 9 { 2u64.pow(13u32) } else { 256 };
    assert_eq!(reconnect_backoff(13), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_14() {
    let transient = [
        "timeout #14",
        "connection reset by peer #14",
        "broken pipe #14",
        "temporarily unavailable #14",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #14",
        "permission denied #14",
        "bad request #14",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_14() {
    let good = format!("Bearer tok_14");
    let lower = format!("bearer tok_14");
    let bad = format!("Basic tok_14");

    assert_eq!(parse_bearer_auth(&good), Some("tok_14"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_14"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_14() {
    let expected = if 14 < 9 { 2u64.pow(14u32) } else { 256 };
    assert_eq!(reconnect_backoff(14), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_15() {
    let transient = [
        "timeout #15",
        "connection reset by peer #15",
        "broken pipe #15",
        "temporarily unavailable #15",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #15",
        "permission denied #15",
        "bad request #15",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_15() {
    let good = format!("Bearer tok_15");
    let lower = format!("bearer tok_15");
    let bad = format!("Basic tok_15");

    assert_eq!(parse_bearer_auth(&good), Some("tok_15"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_15"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_15() {
    let expected = if 15 < 9 { 2u64.pow(15u32) } else { 256 };
    assert_eq!(reconnect_backoff(15), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_16() {
    let transient = [
        "timeout #16",
        "connection reset by peer #16",
        "broken pipe #16",
        "temporarily unavailable #16",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #16",
        "permission denied #16",
        "bad request #16",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_16() {
    let good = format!("Bearer tok_16");
    let lower = format!("bearer tok_16");
    let bad = format!("Basic tok_16");

    assert_eq!(parse_bearer_auth(&good), Some("tok_16"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_16"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_16() {
    let expected = if 16 < 9 { 2u64.pow(16u32) } else { 256 };
    assert_eq!(reconnect_backoff(16), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_17() {
    let transient = [
        "timeout #17",
        "connection reset by peer #17",
        "broken pipe #17",
        "temporarily unavailable #17",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #17",
        "permission denied #17",
        "bad request #17",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_17() {
    let good = format!("Bearer tok_17");
    let lower = format!("bearer tok_17");
    let bad = format!("Basic tok_17");

    assert_eq!(parse_bearer_auth(&good), Some("tok_17"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_17"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_17() {
    let expected = if 17 < 9 { 2u64.pow(17u32) } else { 256 };
    assert_eq!(reconnect_backoff(17), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_18() {
    let transient = [
        "timeout #18",
        "connection reset by peer #18",
        "broken pipe #18",
        "temporarily unavailable #18",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #18",
        "permission denied #18",
        "bad request #18",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_18() {
    let good = format!("Bearer tok_18");
    let lower = format!("bearer tok_18");
    let bad = format!("Basic tok_18");

    assert_eq!(parse_bearer_auth(&good), Some("tok_18"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_18"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_18() {
    let expected = if 18 < 9 { 2u64.pow(18u32) } else { 256 };
    assert_eq!(reconnect_backoff(18), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_19() {
    let transient = [
        "timeout #19",
        "connection reset by peer #19",
        "broken pipe #19",
        "temporarily unavailable #19",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #19",
        "permission denied #19",
        "bad request #19",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_19() {
    let good = format!("Bearer tok_19");
    let lower = format!("bearer tok_19");
    let bad = format!("Basic tok_19");

    assert_eq!(parse_bearer_auth(&good), Some("tok_19"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_19"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_19() {
    let expected = if 19 < 9 { 2u64.pow(19u32) } else { 256 };
    assert_eq!(reconnect_backoff(19), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_20() {
    let transient = [
        "timeout #20",
        "connection reset by peer #20",
        "broken pipe #20",
        "temporarily unavailable #20",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #20",
        "permission denied #20",
        "bad request #20",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_20() {
    let good = format!("Bearer tok_20");
    let lower = format!("bearer tok_20");
    let bad = format!("Basic tok_20");

    assert_eq!(parse_bearer_auth(&good), Some("tok_20"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_20"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_20() {
    let expected = if 20 < 9 { 2u64.pow(20u32) } else { 256 };
    assert_eq!(reconnect_backoff(20), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_21() {
    let transient = [
        "timeout #21",
        "connection reset by peer #21",
        "broken pipe #21",
        "temporarily unavailable #21",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #21",
        "permission denied #21",
        "bad request #21",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_21() {
    let good = format!("Bearer tok_21");
    let lower = format!("bearer tok_21");
    let bad = format!("Basic tok_21");

    assert_eq!(parse_bearer_auth(&good), Some("tok_21"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_21"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_21() {
    let expected = if 21 < 9 { 2u64.pow(21u32) } else { 256 };
    assert_eq!(reconnect_backoff(21), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_22() {
    let transient = [
        "timeout #22",
        "connection reset by peer #22",
        "broken pipe #22",
        "temporarily unavailable #22",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #22",
        "permission denied #22",
        "bad request #22",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_22() {
    let good = format!("Bearer tok_22");
    let lower = format!("bearer tok_22");
    let bad = format!("Basic tok_22");

    assert_eq!(parse_bearer_auth(&good), Some("tok_22"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_22"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_22() {
    let expected = if 22 < 9 { 2u64.pow(22u32) } else { 256 };
    assert_eq!(reconnect_backoff(22), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_23() {
    let transient = [
        "timeout #23",
        "connection reset by peer #23",
        "broken pipe #23",
        "temporarily unavailable #23",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #23",
        "permission denied #23",
        "bad request #23",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_23() {
    let good = format!("Bearer tok_23");
    let lower = format!("bearer tok_23");
    let bad = format!("Basic tok_23");

    assert_eq!(parse_bearer_auth(&good), Some("tok_23"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_23"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_23() {
    let expected = if 23 < 9 { 2u64.pow(23u32) } else { 256 };
    assert_eq!(reconnect_backoff(23), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_24() {
    let transient = [
        "timeout #24",
        "connection reset by peer #24",
        "broken pipe #24",
        "temporarily unavailable #24",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #24",
        "permission denied #24",
        "bad request #24",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_24() {
    let good = format!("Bearer tok_24");
    let lower = format!("bearer tok_24");
    let bad = format!("Basic tok_24");

    assert_eq!(parse_bearer_auth(&good), Some("tok_24"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_24"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_24() {
    let expected = if 24 < 9 { 2u64.pow(24u32) } else { 256 };
    assert_eq!(reconnect_backoff(24), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_25() {
    let transient = [
        "timeout #25",
        "connection reset by peer #25",
        "broken pipe #25",
        "temporarily unavailable #25",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #25",
        "permission denied #25",
        "bad request #25",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_25() {
    let good = format!("Bearer tok_25");
    let lower = format!("bearer tok_25");
    let bad = format!("Basic tok_25");

    assert_eq!(parse_bearer_auth(&good), Some("tok_25"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_25"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_25() {
    let expected = if 25 < 9 { 2u64.pow(25u32) } else { 256 };
    assert_eq!(reconnect_backoff(25), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_26() {
    let transient = [
        "timeout #26",
        "connection reset by peer #26",
        "broken pipe #26",
        "temporarily unavailable #26",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #26",
        "permission denied #26",
        "bad request #26",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_26() {
    let good = format!("Bearer tok_26");
    let lower = format!("bearer tok_26");
    let bad = format!("Basic tok_26");

    assert_eq!(parse_bearer_auth(&good), Some("tok_26"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_26"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_26() {
    let expected = if 26 < 9 { 2u64.pow(26u32) } else { 256 };
    assert_eq!(reconnect_backoff(26), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_27() {
    let transient = [
        "timeout #27",
        "connection reset by peer #27",
        "broken pipe #27",
        "temporarily unavailable #27",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #27",
        "permission denied #27",
        "bad request #27",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_27() {
    let good = format!("Bearer tok_27");
    let lower = format!("bearer tok_27");
    let bad = format!("Basic tok_27");

    assert_eq!(parse_bearer_auth(&good), Some("tok_27"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_27"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_27() {
    let expected = if 27 < 9 { 2u64.pow(27u32) } else { 256 };
    assert_eq!(reconnect_backoff(27), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_28() {
    let transient = [
        "timeout #28",
        "connection reset by peer #28",
        "broken pipe #28",
        "temporarily unavailable #28",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #28",
        "permission denied #28",
        "bad request #28",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_28() {
    let good = format!("Bearer tok_28");
    let lower = format!("bearer tok_28");
    let bad = format!("Basic tok_28");

    assert_eq!(parse_bearer_auth(&good), Some("tok_28"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_28"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_28() {
    let expected = if 28 < 9 { 2u64.pow(28u32) } else { 256 };
    assert_eq!(reconnect_backoff(28), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_29() {
    let transient = [
        "timeout #29",
        "connection reset by peer #29",
        "broken pipe #29",
        "temporarily unavailable #29",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #29",
        "permission denied #29",
        "bad request #29",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_29() {
    let good = format!("Bearer tok_29");
    let lower = format!("bearer tok_29");
    let bad = format!("Basic tok_29");

    assert_eq!(parse_bearer_auth(&good), Some("tok_29"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_29"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_29() {
    let expected = if 29 < 9 { 2u64.pow(29u32) } else { 256 };
    assert_eq!(reconnect_backoff(29), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_30() {
    let transient = [
        "timeout #30",
        "connection reset by peer #30",
        "broken pipe #30",
        "temporarily unavailable #30",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #30",
        "permission denied #30",
        "bad request #30",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_30() {
    let good = format!("Bearer tok_30");
    let lower = format!("bearer tok_30");
    let bad = format!("Basic tok_30");

    assert_eq!(parse_bearer_auth(&good), Some("tok_30"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_30"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_30() {
    let expected = if 30 < 9 { 2u64.pow(30u32) } else { 256 };
    assert_eq!(reconnect_backoff(30), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_31() {
    let transient = [
        "timeout #31",
        "connection reset by peer #31",
        "broken pipe #31",
        "temporarily unavailable #31",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #31",
        "permission denied #31",
        "bad request #31",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_31() {
    let good = format!("Bearer tok_31");
    let lower = format!("bearer tok_31");
    let bad = format!("Basic tok_31");

    assert_eq!(parse_bearer_auth(&good), Some("tok_31"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_31"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_31() {
    let expected = if 31 < 9 { 2u64.pow(31u32) } else { 256 };
    assert_eq!(reconnect_backoff(31), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_32() {
    let transient = [
        "timeout #32",
        "connection reset by peer #32",
        "broken pipe #32",
        "temporarily unavailable #32",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #32",
        "permission denied #32",
        "bad request #32",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_32() {
    let good = format!("Bearer tok_32");
    let lower = format!("bearer tok_32");
    let bad = format!("Basic tok_32");

    assert_eq!(parse_bearer_auth(&good), Some("tok_32"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_32"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_32() {
    let expected = if 32 < 9 { 2u64.pow(32u32) } else { 256 };
    assert_eq!(reconnect_backoff(32), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_33() {
    let transient = [
        "timeout #33",
        "connection reset by peer #33",
        "broken pipe #33",
        "temporarily unavailable #33",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #33",
        "permission denied #33",
        "bad request #33",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_33() {
    let good = format!("Bearer tok_33");
    let lower = format!("bearer tok_33");
    let bad = format!("Basic tok_33");

    assert_eq!(parse_bearer_auth(&good), Some("tok_33"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_33"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_33() {
    let expected = if 33 < 9 { 2u64.pow(33u32) } else { 256 };
    assert_eq!(reconnect_backoff(33), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_34() {
    let transient = [
        "timeout #34",
        "connection reset by peer #34",
        "broken pipe #34",
        "temporarily unavailable #34",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #34",
        "permission denied #34",
        "bad request #34",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_34() {
    let good = format!("Bearer tok_34");
    let lower = format!("bearer tok_34");
    let bad = format!("Basic tok_34");

    assert_eq!(parse_bearer_auth(&good), Some("tok_34"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_34"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_34() {
    let expected = if 34 < 9 { 2u64.pow(34u32) } else { 256 };
    assert_eq!(reconnect_backoff(34), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_35() {
    let transient = [
        "timeout #35",
        "connection reset by peer #35",
        "broken pipe #35",
        "temporarily unavailable #35",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #35",
        "permission denied #35",
        "bad request #35",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_35() {
    let good = format!("Bearer tok_35");
    let lower = format!("bearer tok_35");
    let bad = format!("Basic tok_35");

    assert_eq!(parse_bearer_auth(&good), Some("tok_35"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_35"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_35() {
    let expected = if 35 < 9 { 2u64.pow(35u32) } else { 256 };
    assert_eq!(reconnect_backoff(35), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_36() {
    let transient = [
        "timeout #36",
        "connection reset by peer #36",
        "broken pipe #36",
        "temporarily unavailable #36",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #36",
        "permission denied #36",
        "bad request #36",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_36() {
    let good = format!("Bearer tok_36");
    let lower = format!("bearer tok_36");
    let bad = format!("Basic tok_36");

    assert_eq!(parse_bearer_auth(&good), Some("tok_36"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_36"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_36() {
    let expected = if 36 < 9 { 2u64.pow(36u32) } else { 256 };
    assert_eq!(reconnect_backoff(36), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_37() {
    let transient = [
        "timeout #37",
        "connection reset by peer #37",
        "broken pipe #37",
        "temporarily unavailable #37",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #37",
        "permission denied #37",
        "bad request #37",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_37() {
    let good = format!("Bearer tok_37");
    let lower = format!("bearer tok_37");
    let bad = format!("Basic tok_37");

    assert_eq!(parse_bearer_auth(&good), Some("tok_37"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_37"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_37() {
    let expected = if 37 < 9 { 2u64.pow(37u32) } else { 256 };
    assert_eq!(reconnect_backoff(37), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_38() {
    let transient = [
        "timeout #38",
        "connection reset by peer #38",
        "broken pipe #38",
        "temporarily unavailable #38",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #38",
        "permission denied #38",
        "bad request #38",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_38() {
    let good = format!("Bearer tok_38");
    let lower = format!("bearer tok_38");
    let bad = format!("Basic tok_38");

    assert_eq!(parse_bearer_auth(&good), Some("tok_38"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_38"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_38() {
    let expected = if 38 < 9 { 2u64.pow(38u32) } else { 256 };
    assert_eq!(reconnect_backoff(38), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_39() {
    let transient = [
        "timeout #39",
        "connection reset by peer #39",
        "broken pipe #39",
        "temporarily unavailable #39",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #39",
        "permission denied #39",
        "bad request #39",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_39() {
    let good = format!("Bearer tok_39");
    let lower = format!("bearer tok_39");
    let bad = format!("Basic tok_39");

    assert_eq!(parse_bearer_auth(&good), Some("tok_39"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_39"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_39() {
    let expected = if 39 < 9 { 2u64.pow(39u32) } else { 256 };
    assert_eq!(reconnect_backoff(39), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_40() {
    let transient = [
        "timeout #40",
        "connection reset by peer #40",
        "broken pipe #40",
        "temporarily unavailable #40",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #40",
        "permission denied #40",
        "bad request #40",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_40() {
    let good = format!("Bearer tok_40");
    let lower = format!("bearer tok_40");
    let bad = format!("Basic tok_40");

    assert_eq!(parse_bearer_auth(&good), Some("tok_40"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_40"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_40() {
    let expected = if 40 < 9 { 2u64.pow(40u32) } else { 256 };
    assert_eq!(reconnect_backoff(40), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_41() {
    let transient = [
        "timeout #41",
        "connection reset by peer #41",
        "broken pipe #41",
        "temporarily unavailable #41",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #41",
        "permission denied #41",
        "bad request #41",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_41() {
    let good = format!("Bearer tok_41");
    let lower = format!("bearer tok_41");
    let bad = format!("Basic tok_41");

    assert_eq!(parse_bearer_auth(&good), Some("tok_41"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_41"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_41() {
    let expected = if 41 < 9 { 2u64.pow(41u32) } else { 256 };
    assert_eq!(reconnect_backoff(41), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_42() {
    let transient = [
        "timeout #42",
        "connection reset by peer #42",
        "broken pipe #42",
        "temporarily unavailable #42",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #42",
        "permission denied #42",
        "bad request #42",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_42() {
    let good = format!("Bearer tok_42");
    let lower = format!("bearer tok_42");
    let bad = format!("Basic tok_42");

    assert_eq!(parse_bearer_auth(&good), Some("tok_42"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_42"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_42() {
    let expected = if 42 < 9 { 2u64.pow(42u32) } else { 256 };
    assert_eq!(reconnect_backoff(42), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_43() {
    let transient = [
        "timeout #43",
        "connection reset by peer #43",
        "broken pipe #43",
        "temporarily unavailable #43",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #43",
        "permission denied #43",
        "bad request #43",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_43() {
    let good = format!("Bearer tok_43");
    let lower = format!("bearer tok_43");
    let bad = format!("Basic tok_43");

    assert_eq!(parse_bearer_auth(&good), Some("tok_43"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_43"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_43() {
    let expected = if 43 < 9 { 2u64.pow(43u32) } else { 256 };
    assert_eq!(reconnect_backoff(43), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_44() {
    let transient = [
        "timeout #44",
        "connection reset by peer #44",
        "broken pipe #44",
        "temporarily unavailable #44",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #44",
        "permission denied #44",
        "bad request #44",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_44() {
    let good = format!("Bearer tok_44");
    let lower = format!("bearer tok_44");
    let bad = format!("Basic tok_44");

    assert_eq!(parse_bearer_auth(&good), Some("tok_44"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_44"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_44() {
    let expected = if 44 < 9 { 2u64.pow(44u32) } else { 256 };
    assert_eq!(reconnect_backoff(44), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_45() {
    let transient = [
        "timeout #45",
        "connection reset by peer #45",
        "broken pipe #45",
        "temporarily unavailable #45",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #45",
        "permission denied #45",
        "bad request #45",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_45() {
    let good = format!("Bearer tok_45");
    let lower = format!("bearer tok_45");
    let bad = format!("Basic tok_45");

    assert_eq!(parse_bearer_auth(&good), Some("tok_45"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_45"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_45() {
    let expected = if 45 < 9 { 2u64.pow(45u32) } else { 256 };
    assert_eq!(reconnect_backoff(45), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_46() {
    let transient = [
        "timeout #46",
        "connection reset by peer #46",
        "broken pipe #46",
        "temporarily unavailable #46",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #46",
        "permission denied #46",
        "bad request #46",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_46() {
    let good = format!("Bearer tok_46");
    let lower = format!("bearer tok_46");
    let bad = format!("Basic tok_46");

    assert_eq!(parse_bearer_auth(&good), Some("tok_46"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_46"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_46() {
    let expected = if 46 < 9 { 2u64.pow(46u32) } else { 256 };
    assert_eq!(reconnect_backoff(46), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_47() {
    let transient = [
        "timeout #47",
        "connection reset by peer #47",
        "broken pipe #47",
        "temporarily unavailable #47",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #47",
        "permission denied #47",
        "bad request #47",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_47() {
    let good = format!("Bearer tok_47");
    let lower = format!("bearer tok_47");
    let bad = format!("Basic tok_47");

    assert_eq!(parse_bearer_auth(&good), Some("tok_47"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_47"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_47() {
    let expected = if 47 < 9 { 2u64.pow(47u32) } else { 256 };
    assert_eq!(reconnect_backoff(47), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_48() {
    let transient = [
        "timeout #48",
        "connection reset by peer #48",
        "broken pipe #48",
        "temporarily unavailable #48",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #48",
        "permission denied #48",
        "bad request #48",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_48() {
    let good = format!("Bearer tok_48");
    let lower = format!("bearer tok_48");
    let bad = format!("Basic tok_48");

    assert_eq!(parse_bearer_auth(&good), Some("tok_48"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_48"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_48() {
    let expected = if 48 < 9 { 2u64.pow(48u32) } else { 256 };
    assert_eq!(reconnect_backoff(48), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_49() {
    let transient = [
        "timeout #49",
        "connection reset by peer #49",
        "broken pipe #49",
        "temporarily unavailable #49",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #49",
        "permission denied #49",
        "bad request #49",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_49() {
    let good = format!("Bearer tok_49");
    let lower = format!("bearer tok_49");
    let bad = format!("Basic tok_49");

    assert_eq!(parse_bearer_auth(&good), Some("tok_49"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_49"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_49() {
    let expected = if 49 < 9 { 2u64.pow(49u32) } else { 256 };
    assert_eq!(reconnect_backoff(49), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_50() {
    let transient = [
        "timeout #50",
        "connection reset by peer #50",
        "broken pipe #50",
        "temporarily unavailable #50",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #50",
        "permission denied #50",
        "bad request #50",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_50() {
    let good = format!("Bearer tok_50");
    let lower = format!("bearer tok_50");
    let bad = format!("Basic tok_50");

    assert_eq!(parse_bearer_auth(&good), Some("tok_50"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_50"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_50() {
    let expected = if 50 < 9 { 2u64.pow(50u32) } else { 256 };
    assert_eq!(reconnect_backoff(50), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_51() {
    let transient = [
        "timeout #51",
        "connection reset by peer #51",
        "broken pipe #51",
        "temporarily unavailable #51",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #51",
        "permission denied #51",
        "bad request #51",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_51() {
    let good = format!("Bearer tok_51");
    let lower = format!("bearer tok_51");
    let bad = format!("Basic tok_51");

    assert_eq!(parse_bearer_auth(&good), Some("tok_51"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_51"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_51() {
    let expected = if 51 < 9 { 2u64.pow(51u32) } else { 256 };
    assert_eq!(reconnect_backoff(51), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_52() {
    let transient = [
        "timeout #52",
        "connection reset by peer #52",
        "broken pipe #52",
        "temporarily unavailable #52",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #52",
        "permission denied #52",
        "bad request #52",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_52() {
    let good = format!("Bearer tok_52");
    let lower = format!("bearer tok_52");
    let bad = format!("Basic tok_52");

    assert_eq!(parse_bearer_auth(&good), Some("tok_52"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_52"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_52() {
    let expected = if 52 < 9 { 2u64.pow(52u32) } else { 256 };
    assert_eq!(reconnect_backoff(52), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_53() {
    let transient = [
        "timeout #53",
        "connection reset by peer #53",
        "broken pipe #53",
        "temporarily unavailable #53",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #53",
        "permission denied #53",
        "bad request #53",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_53() {
    let good = format!("Bearer tok_53");
    let lower = format!("bearer tok_53");
    let bad = format!("Basic tok_53");

    assert_eq!(parse_bearer_auth(&good), Some("tok_53"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_53"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_53() {
    let expected = if 53 < 9 { 2u64.pow(53u32) } else { 256 };
    assert_eq!(reconnect_backoff(53), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_54() {
    let transient = [
        "timeout #54",
        "connection reset by peer #54",
        "broken pipe #54",
        "temporarily unavailable #54",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #54",
        "permission denied #54",
        "bad request #54",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_54() {
    let good = format!("Bearer tok_54");
    let lower = format!("bearer tok_54");
    let bad = format!("Basic tok_54");

    assert_eq!(parse_bearer_auth(&good), Some("tok_54"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_54"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_54() {
    let expected = if 54 < 9 { 2u64.pow(54u32) } else { 256 };
    assert_eq!(reconnect_backoff(54), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_55() {
    let transient = [
        "timeout #55",
        "connection reset by peer #55",
        "broken pipe #55",
        "temporarily unavailable #55",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #55",
        "permission denied #55",
        "bad request #55",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_55() {
    let good = format!("Bearer tok_55");
    let lower = format!("bearer tok_55");
    let bad = format!("Basic tok_55");

    assert_eq!(parse_bearer_auth(&good), Some("tok_55"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_55"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_55() {
    let expected = if 55 < 9 { 2u64.pow(55u32) } else { 256 };
    assert_eq!(reconnect_backoff(55), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_56() {
    let transient = [
        "timeout #56",
        "connection reset by peer #56",
        "broken pipe #56",
        "temporarily unavailable #56",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #56",
        "permission denied #56",
        "bad request #56",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_56() {
    let good = format!("Bearer tok_56");
    let lower = format!("bearer tok_56");
    let bad = format!("Basic tok_56");

    assert_eq!(parse_bearer_auth(&good), Some("tok_56"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_56"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_56() {
    let expected = if 56 < 9 { 2u64.pow(56u32) } else { 256 };
    assert_eq!(reconnect_backoff(56), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_57() {
    let transient = [
        "timeout #57",
        "connection reset by peer #57",
        "broken pipe #57",
        "temporarily unavailable #57",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #57",
        "permission denied #57",
        "bad request #57",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_57() {
    let good = format!("Bearer tok_57");
    let lower = format!("bearer tok_57");
    let bad = format!("Basic tok_57");

    assert_eq!(parse_bearer_auth(&good), Some("tok_57"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_57"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_57() {
    let expected = if 57 < 9 { 2u64.pow(57u32) } else { 256 };
    assert_eq!(reconnect_backoff(57), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_58() {
    let transient = [
        "timeout #58",
        "connection reset by peer #58",
        "broken pipe #58",
        "temporarily unavailable #58",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #58",
        "permission denied #58",
        "bad request #58",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_58() {
    let good = format!("Bearer tok_58");
    let lower = format!("bearer tok_58");
    let bad = format!("Basic tok_58");

    assert_eq!(parse_bearer_auth(&good), Some("tok_58"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_58"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_58() {
    let expected = if 58 < 9 { 2u64.pow(58u32) } else { 256 };
    assert_eq!(reconnect_backoff(58), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_59() {
    let transient = [
        "timeout #59",
        "connection reset by peer #59",
        "broken pipe #59",
        "temporarily unavailable #59",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #59",
        "permission denied #59",
        "bad request #59",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_59() {
    let good = format!("Bearer tok_59");
    let lower = format!("bearer tok_59");
    let bad = format!("Basic tok_59");

    assert_eq!(parse_bearer_auth(&good), Some("tok_59"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_59"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_59() {
    let expected = if 59 < 9 { 2u64.pow(59u32) } else { 256 };
    assert_eq!(reconnect_backoff(59), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_60() {
    let transient = [
        "timeout #60",
        "connection reset by peer #60",
        "broken pipe #60",
        "temporarily unavailable #60",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #60",
        "permission denied #60",
        "bad request #60",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_60() {
    let good = format!("Bearer tok_60");
    let lower = format!("bearer tok_60");
    let bad = format!("Basic tok_60");

    assert_eq!(parse_bearer_auth(&good), Some("tok_60"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_60"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_60() {
    let expected = if 60 < 9 { 2u64.pow(60u32) } else { 256 };
    assert_eq!(reconnect_backoff(60), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_61() {
    let transient = [
        "timeout #61",
        "connection reset by peer #61",
        "broken pipe #61",
        "temporarily unavailable #61",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #61",
        "permission denied #61",
        "bad request #61",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_61() {
    let good = format!("Bearer tok_61");
    let lower = format!("bearer tok_61");
    let bad = format!("Basic tok_61");

    assert_eq!(parse_bearer_auth(&good), Some("tok_61"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_61"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_61() {
    let expected = if 61 < 9 { 2u64.pow(61u32) } else { 256 };
    assert_eq!(reconnect_backoff(61), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_62() {
    let transient = [
        "timeout #62",
        "connection reset by peer #62",
        "broken pipe #62",
        "temporarily unavailable #62",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #62",
        "permission denied #62",
        "bad request #62",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_62() {
    let good = format!("Bearer tok_62");
    let lower = format!("bearer tok_62");
    let bad = format!("Basic tok_62");

    assert_eq!(parse_bearer_auth(&good), Some("tok_62"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_62"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_62() {
    let expected = if 62 < 9 { 2u64.pow(62u32) } else { 256 };
    assert_eq!(reconnect_backoff(62), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_63() {
    let transient = [
        "timeout #63",
        "connection reset by peer #63",
        "broken pipe #63",
        "temporarily unavailable #63",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #63",
        "permission denied #63",
        "bad request #63",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_63() {
    let good = format!("Bearer tok_63");
    let lower = format!("bearer tok_63");
    let bad = format!("Basic tok_63");

    assert_eq!(parse_bearer_auth(&good), Some("tok_63"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_63"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_63() {
    let expected = if 63 < 9 { 2u64.pow(63u32) } else { 256 };
    assert_eq!(reconnect_backoff(63), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_64() {
    let transient = [
        "timeout #64",
        "connection reset by peer #64",
        "broken pipe #64",
        "temporarily unavailable #64",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #64",
        "permission denied #64",
        "bad request #64",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_64() {
    let good = format!("Bearer tok_64");
    let lower = format!("bearer tok_64");
    let bad = format!("Basic tok_64");

    assert_eq!(parse_bearer_auth(&good), Some("tok_64"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_64"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_64() {
    let expected = if 64 < 9 { 2u64.pow(64u32) } else { 256 };
    assert_eq!(reconnect_backoff(64), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_65() {
    let transient = [
        "timeout #65",
        "connection reset by peer #65",
        "broken pipe #65",
        "temporarily unavailable #65",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #65",
        "permission denied #65",
        "bad request #65",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_65() {
    let good = format!("Bearer tok_65");
    let lower = format!("bearer tok_65");
    let bad = format!("Basic tok_65");

    assert_eq!(parse_bearer_auth(&good), Some("tok_65"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_65"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_65() {
    let expected = if 65 < 9 { 2u64.pow(65u32) } else { 256 };
    assert_eq!(reconnect_backoff(65), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_66() {
    let transient = [
        "timeout #66",
        "connection reset by peer #66",
        "broken pipe #66",
        "temporarily unavailable #66",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #66",
        "permission denied #66",
        "bad request #66",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_66() {
    let good = format!("Bearer tok_66");
    let lower = format!("bearer tok_66");
    let bad = format!("Basic tok_66");

    assert_eq!(parse_bearer_auth(&good), Some("tok_66"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_66"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_66() {
    let expected = if 66 < 9 { 2u64.pow(66u32) } else { 256 };
    assert_eq!(reconnect_backoff(66), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_67() {
    let transient = [
        "timeout #67",
        "connection reset by peer #67",
        "broken pipe #67",
        "temporarily unavailable #67",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #67",
        "permission denied #67",
        "bad request #67",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_67() {
    let good = format!("Bearer tok_67");
    let lower = format!("bearer tok_67");
    let bad = format!("Basic tok_67");

    assert_eq!(parse_bearer_auth(&good), Some("tok_67"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_67"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_67() {
    let expected = if 67 < 9 { 2u64.pow(67u32) } else { 256 };
    assert_eq!(reconnect_backoff(67), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_68() {
    let transient = [
        "timeout #68",
        "connection reset by peer #68",
        "broken pipe #68",
        "temporarily unavailable #68",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #68",
        "permission denied #68",
        "bad request #68",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_68() {
    let good = format!("Bearer tok_68");
    let lower = format!("bearer tok_68");
    let bad = format!("Basic tok_68");

    assert_eq!(parse_bearer_auth(&good), Some("tok_68"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_68"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_68() {
    let expected = if 68 < 9 { 2u64.pow(68u32) } else { 256 };
    assert_eq!(reconnect_backoff(68), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_69() {
    let transient = [
        "timeout #69",
        "connection reset by peer #69",
        "broken pipe #69",
        "temporarily unavailable #69",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #69",
        "permission denied #69",
        "bad request #69",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_69() {
    let good = format!("Bearer tok_69");
    let lower = format!("bearer tok_69");
    let bad = format!("Basic tok_69");

    assert_eq!(parse_bearer_auth(&good), Some("tok_69"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_69"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_69() {
    let expected = if 69 < 9 { 2u64.pow(69u32) } else { 256 };
    assert_eq!(reconnect_backoff(69), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_70() {
    let transient = [
        "timeout #70",
        "connection reset by peer #70",
        "broken pipe #70",
        "temporarily unavailable #70",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #70",
        "permission denied #70",
        "bad request #70",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_70() {
    let good = format!("Bearer tok_70");
    let lower = format!("bearer tok_70");
    let bad = format!("Basic tok_70");

    assert_eq!(parse_bearer_auth(&good), Some("tok_70"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_70"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_70() {
    let expected = if 70 < 9 { 2u64.pow(70u32) } else { 256 };
    assert_eq!(reconnect_backoff(70), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_71() {
    let transient = [
        "timeout #71",
        "connection reset by peer #71",
        "broken pipe #71",
        "temporarily unavailable #71",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #71",
        "permission denied #71",
        "bad request #71",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_71() {
    let good = format!("Bearer tok_71");
    let lower = format!("bearer tok_71");
    let bad = format!("Basic tok_71");

    assert_eq!(parse_bearer_auth(&good), Some("tok_71"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_71"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_71() {
    let expected = if 71 < 9 { 2u64.pow(71u32) } else { 256 };
    assert_eq!(reconnect_backoff(71), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_72() {
    let transient = [
        "timeout #72",
        "connection reset by peer #72",
        "broken pipe #72",
        "temporarily unavailable #72",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #72",
        "permission denied #72",
        "bad request #72",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_72() {
    let good = format!("Bearer tok_72");
    let lower = format!("bearer tok_72");
    let bad = format!("Basic tok_72");

    assert_eq!(parse_bearer_auth(&good), Some("tok_72"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_72"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_72() {
    let expected = if 72 < 9 { 2u64.pow(72u32) } else { 256 };
    assert_eq!(reconnect_backoff(72), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_73() {
    let transient = [
        "timeout #73",
        "connection reset by peer #73",
        "broken pipe #73",
        "temporarily unavailable #73",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #73",
        "permission denied #73",
        "bad request #73",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_73() {
    let good = format!("Bearer tok_73");
    let lower = format!("bearer tok_73");
    let bad = format!("Basic tok_73");

    assert_eq!(parse_bearer_auth(&good), Some("tok_73"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_73"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_73() {
    let expected = if 73 < 9 { 2u64.pow(73u32) } else { 256 };
    assert_eq!(reconnect_backoff(73), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_74() {
    let transient = [
        "timeout #74",
        "connection reset by peer #74",
        "broken pipe #74",
        "temporarily unavailable #74",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #74",
        "permission denied #74",
        "bad request #74",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_74() {
    let good = format!("Bearer tok_74");
    let lower = format!("bearer tok_74");
    let bad = format!("Basic tok_74");

    assert_eq!(parse_bearer_auth(&good), Some("tok_74"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_74"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_74() {
    let expected = if 74 < 9 { 2u64.pow(74u32) } else { 256 };
    assert_eq!(reconnect_backoff(74), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_75() {
    let transient = [
        "timeout #75",
        "connection reset by peer #75",
        "broken pipe #75",
        "temporarily unavailable #75",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #75",
        "permission denied #75",
        "bad request #75",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_75() {
    let good = format!("Bearer tok_75");
    let lower = format!("bearer tok_75");
    let bad = format!("Basic tok_75");

    assert_eq!(parse_bearer_auth(&good), Some("tok_75"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_75"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_75() {
    let expected = if 75 < 9 { 2u64.pow(75u32) } else { 256 };
    assert_eq!(reconnect_backoff(75), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_76() {
    let transient = [
        "timeout #76",
        "connection reset by peer #76",
        "broken pipe #76",
        "temporarily unavailable #76",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #76",
        "permission denied #76",
        "bad request #76",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_76() {
    let good = format!("Bearer tok_76");
    let lower = format!("bearer tok_76");
    let bad = format!("Basic tok_76");

    assert_eq!(parse_bearer_auth(&good), Some("tok_76"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_76"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_76() {
    let expected = if 76 < 9 { 2u64.pow(76u32) } else { 256 };
    assert_eq!(reconnect_backoff(76), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_77() {
    let transient = [
        "timeout #77",
        "connection reset by peer #77",
        "broken pipe #77",
        "temporarily unavailable #77",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #77",
        "permission denied #77",
        "bad request #77",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_77() {
    let good = format!("Bearer tok_77");
    let lower = format!("bearer tok_77");
    let bad = format!("Basic tok_77");

    assert_eq!(parse_bearer_auth(&good), Some("tok_77"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_77"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_77() {
    let expected = if 77 < 9 { 2u64.pow(77u32) } else { 256 };
    assert_eq!(reconnect_backoff(77), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_78() {
    let transient = [
        "timeout #78",
        "connection reset by peer #78",
        "broken pipe #78",
        "temporarily unavailable #78",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #78",
        "permission denied #78",
        "bad request #78",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_78() {
    let good = format!("Bearer tok_78");
    let lower = format!("bearer tok_78");
    let bad = format!("Basic tok_78");

    assert_eq!(parse_bearer_auth(&good), Some("tok_78"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_78"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_78() {
    let expected = if 78 < 9 { 2u64.pow(78u32) } else { 256 };
    assert_eq!(reconnect_backoff(78), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_79() {
    let transient = [
        "timeout #79",
        "connection reset by peer #79",
        "broken pipe #79",
        "temporarily unavailable #79",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #79",
        "permission denied #79",
        "bad request #79",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_79() {
    let good = format!("Bearer tok_79");
    let lower = format!("bearer tok_79");
    let bad = format!("Basic tok_79");

    assert_eq!(parse_bearer_auth(&good), Some("tok_79"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_79"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_79() {
    let expected = if 79 < 9 { 2u64.pow(79u32) } else { 256 };
    assert_eq!(reconnect_backoff(79), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_80() {
    let transient = [
        "timeout #80",
        "connection reset by peer #80",
        "broken pipe #80",
        "temporarily unavailable #80",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #80",
        "permission denied #80",
        "bad request #80",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_80() {
    let good = format!("Bearer tok_80");
    let lower = format!("bearer tok_80");
    let bad = format!("Basic tok_80");

    assert_eq!(parse_bearer_auth(&good), Some("tok_80"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_80"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_80() {
    let expected = if 80 < 9 { 2u64.pow(80u32) } else { 256 };
    assert_eq!(reconnect_backoff(80), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_81() {
    let transient = [
        "timeout #81",
        "connection reset by peer #81",
        "broken pipe #81",
        "temporarily unavailable #81",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #81",
        "permission denied #81",
        "bad request #81",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_81() {
    let good = format!("Bearer tok_81");
    let lower = format!("bearer tok_81");
    let bad = format!("Basic tok_81");

    assert_eq!(parse_bearer_auth(&good), Some("tok_81"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_81"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_81() {
    let expected = if 81 < 9 { 2u64.pow(81u32) } else { 256 };
    assert_eq!(reconnect_backoff(81), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_82() {
    let transient = [
        "timeout #82",
        "connection reset by peer #82",
        "broken pipe #82",
        "temporarily unavailable #82",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #82",
        "permission denied #82",
        "bad request #82",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_82() {
    let good = format!("Bearer tok_82");
    let lower = format!("bearer tok_82");
    let bad = format!("Basic tok_82");

    assert_eq!(parse_bearer_auth(&good), Some("tok_82"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_82"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_82() {
    let expected = if 82 < 9 { 2u64.pow(82u32) } else { 256 };
    assert_eq!(reconnect_backoff(82), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_83() {
    let transient = [
        "timeout #83",
        "connection reset by peer #83",
        "broken pipe #83",
        "temporarily unavailable #83",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #83",
        "permission denied #83",
        "bad request #83",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_83() {
    let good = format!("Bearer tok_83");
    let lower = format!("bearer tok_83");
    let bad = format!("Basic tok_83");

    assert_eq!(parse_bearer_auth(&good), Some("tok_83"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_83"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_83() {
    let expected = if 83 < 9 { 2u64.pow(83u32) } else { 256 };
    assert_eq!(reconnect_backoff(83), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_84() {
    let transient = [
        "timeout #84",
        "connection reset by peer #84",
        "broken pipe #84",
        "temporarily unavailable #84",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #84",
        "permission denied #84",
        "bad request #84",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_84() {
    let good = format!("Bearer tok_84");
    let lower = format!("bearer tok_84");
    let bad = format!("Basic tok_84");

    assert_eq!(parse_bearer_auth(&good), Some("tok_84"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_84"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_84() {
    let expected = if 84 < 9 { 2u64.pow(84u32) } else { 256 };
    assert_eq!(reconnect_backoff(84), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_85() {
    let transient = [
        "timeout #85",
        "connection reset by peer #85",
        "broken pipe #85",
        "temporarily unavailable #85",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #85",
        "permission denied #85",
        "bad request #85",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_85() {
    let good = format!("Bearer tok_85");
    let lower = format!("bearer tok_85");
    let bad = format!("Basic tok_85");

    assert_eq!(parse_bearer_auth(&good), Some("tok_85"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_85"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_85() {
    let expected = if 85 < 9 { 2u64.pow(85u32) } else { 256 };
    assert_eq!(reconnect_backoff(85), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_86() {
    let transient = [
        "timeout #86",
        "connection reset by peer #86",
        "broken pipe #86",
        "temporarily unavailable #86",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #86",
        "permission denied #86",
        "bad request #86",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_86() {
    let good = format!("Bearer tok_86");
    let lower = format!("bearer tok_86");
    let bad = format!("Basic tok_86");

    assert_eq!(parse_bearer_auth(&good), Some("tok_86"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_86"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_86() {
    let expected = if 86 < 9 { 2u64.pow(86u32) } else { 256 };
    assert_eq!(reconnect_backoff(86), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_87() {
    let transient = [
        "timeout #87",
        "connection reset by peer #87",
        "broken pipe #87",
        "temporarily unavailable #87",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #87",
        "permission denied #87",
        "bad request #87",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_87() {
    let good = format!("Bearer tok_87");
    let lower = format!("bearer tok_87");
    let bad = format!("Basic tok_87");

    assert_eq!(parse_bearer_auth(&good), Some("tok_87"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_87"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_87() {
    let expected = if 87 < 9 { 2u64.pow(87u32) } else { 256 };
    assert_eq!(reconnect_backoff(87), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_88() {
    let transient = [
        "timeout #88",
        "connection reset by peer #88",
        "broken pipe #88",
        "temporarily unavailable #88",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #88",
        "permission denied #88",
        "bad request #88",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_88() {
    let good = format!("Bearer tok_88");
    let lower = format!("bearer tok_88");
    let bad = format!("Basic tok_88");

    assert_eq!(parse_bearer_auth(&good), Some("tok_88"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_88"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_88() {
    let expected = if 88 < 9 { 2u64.pow(88u32) } else { 256 };
    assert_eq!(reconnect_backoff(88), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_89() {
    let transient = [
        "timeout #89",
        "connection reset by peer #89",
        "broken pipe #89",
        "temporarily unavailable #89",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #89",
        "permission denied #89",
        "bad request #89",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_89() {
    let good = format!("Bearer tok_89");
    let lower = format!("bearer tok_89");
    let bad = format!("Basic tok_89");

    assert_eq!(parse_bearer_auth(&good), Some("tok_89"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_89"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_89() {
    let expected = if 89 < 9 { 2u64.pow(89u32) } else { 256 };
    assert_eq!(reconnect_backoff(89), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_90() {
    let transient = [
        "timeout #90",
        "connection reset by peer #90",
        "broken pipe #90",
        "temporarily unavailable #90",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #90",
        "permission denied #90",
        "bad request #90",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_90() {
    let good = format!("Bearer tok_90");
    let lower = format!("bearer tok_90");
    let bad = format!("Basic tok_90");

    assert_eq!(parse_bearer_auth(&good), Some("tok_90"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_90"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_90() {
    let expected = if 90 < 9 { 2u64.pow(90u32) } else { 256 };
    assert_eq!(reconnect_backoff(90), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_91() {
    let transient = [
        "timeout #91",
        "connection reset by peer #91",
        "broken pipe #91",
        "temporarily unavailable #91",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #91",
        "permission denied #91",
        "bad request #91",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_91() {
    let good = format!("Bearer tok_91");
    let lower = format!("bearer tok_91");
    let bad = format!("Basic tok_91");

    assert_eq!(parse_bearer_auth(&good), Some("tok_91"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_91"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_91() {
    let expected = if 91 < 9 { 2u64.pow(91u32) } else { 256 };
    assert_eq!(reconnect_backoff(91), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_92() {
    let transient = [
        "timeout #92",
        "connection reset by peer #92",
        "broken pipe #92",
        "temporarily unavailable #92",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #92",
        "permission denied #92",
        "bad request #92",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_92() {
    let good = format!("Bearer tok_92");
    let lower = format!("bearer tok_92");
    let bad = format!("Basic tok_92");

    assert_eq!(parse_bearer_auth(&good), Some("tok_92"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_92"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_92() {
    let expected = if 92 < 9 { 2u64.pow(92u32) } else { 256 };
    assert_eq!(reconnect_backoff(92), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_93() {
    let transient = [
        "timeout #93",
        "connection reset by peer #93",
        "broken pipe #93",
        "temporarily unavailable #93",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #93",
        "permission denied #93",
        "bad request #93",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_93() {
    let good = format!("Bearer tok_93");
    let lower = format!("bearer tok_93");
    let bad = format!("Basic tok_93");

    assert_eq!(parse_bearer_auth(&good), Some("tok_93"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_93"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_93() {
    let expected = if 93 < 9 { 2u64.pow(93u32) } else { 256 };
    assert_eq!(reconnect_backoff(93), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_94() {
    let transient = [
        "timeout #94",
        "connection reset by peer #94",
        "broken pipe #94",
        "temporarily unavailable #94",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #94",
        "permission denied #94",
        "bad request #94",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_94() {
    let good = format!("Bearer tok_94");
    let lower = format!("bearer tok_94");
    let bad = format!("Basic tok_94");

    assert_eq!(parse_bearer_auth(&good), Some("tok_94"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_94"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_94() {
    let expected = if 94 < 9 { 2u64.pow(94u32) } else { 256 };
    assert_eq!(reconnect_backoff(94), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_95() {
    let transient = [
        "timeout #95",
        "connection reset by peer #95",
        "broken pipe #95",
        "temporarily unavailable #95",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #95",
        "permission denied #95",
        "bad request #95",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_95() {
    let good = format!("Bearer tok_95");
    let lower = format!("bearer tok_95");
    let bad = format!("Basic tok_95");

    assert_eq!(parse_bearer_auth(&good), Some("tok_95"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_95"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_95() {
    let expected = if 95 < 9 { 2u64.pow(95u32) } else { 256 };
    assert_eq!(reconnect_backoff(95), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_96() {
    let transient = [
        "timeout #96",
        "connection reset by peer #96",
        "broken pipe #96",
        "temporarily unavailable #96",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #96",
        "permission denied #96",
        "bad request #96",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_96() {
    let good = format!("Bearer tok_96");
    let lower = format!("bearer tok_96");
    let bad = format!("Basic tok_96");

    assert_eq!(parse_bearer_auth(&good), Some("tok_96"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_96"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_96() {
    let expected = if 96 < 9 { 2u64.pow(96u32) } else { 256 };
    assert_eq!(reconnect_backoff(96), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_97() {
    let transient = [
        "timeout #97",
        "connection reset by peer #97",
        "broken pipe #97",
        "temporarily unavailable #97",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #97",
        "permission denied #97",
        "bad request #97",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_97() {
    let good = format!("Bearer tok_97");
    let lower = format!("bearer tok_97");
    let bad = format!("Basic tok_97");

    assert_eq!(parse_bearer_auth(&good), Some("tok_97"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_97"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_97() {
    let expected = if 97 < 9 { 2u64.pow(97u32) } else { 256 };
    assert_eq!(reconnect_backoff(97), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_98() {
    let transient = [
        "timeout #98",
        "connection reset by peer #98",
        "broken pipe #98",
        "temporarily unavailable #98",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #98",
        "permission denied #98",
        "bad request #98",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_98() {
    let good = format!("Bearer tok_98");
    let lower = format!("bearer tok_98");
    let bad = format!("Basic tok_98");

    assert_eq!(parse_bearer_auth(&good), Some("tok_98"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_98"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_98() {
    let expected = if 98 < 9 { 2u64.pow(98u32) } else { 256 };
    assert_eq!(reconnect_backoff(98), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_99() {
    let transient = [
        "timeout #99",
        "connection reset by peer #99",
        "broken pipe #99",
        "temporarily unavailable #99",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #99",
        "permission denied #99",
        "bad request #99",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_99() {
    let good = format!("Bearer tok_99");
    let lower = format!("bearer tok_99");
    let bad = format!("Basic tok_99");

    assert_eq!(parse_bearer_auth(&good), Some("tok_99"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_99"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_99() {
    let expected = if 99 < 9 { 2u64.pow(99u32) } else { 256 };
    assert_eq!(reconnect_backoff(99), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_100() {
    let transient = [
        "timeout #100",
        "connection reset by peer #100",
        "broken pipe #100",
        "temporarily unavailable #100",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #100",
        "permission denied #100",
        "bad request #100",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_100() {
    let good = format!("Bearer tok_100");
    let lower = format!("bearer tok_100");
    let bad = format!("Basic tok_100");

    assert_eq!(parse_bearer_auth(&good), Some("tok_100"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_100"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_100() {
    let expected = if 100 < 9 { 2u64.pow(100u32) } else { 256 };
    assert_eq!(reconnect_backoff(100), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_101() {
    let transient = [
        "timeout #101",
        "connection reset by peer #101",
        "broken pipe #101",
        "temporarily unavailable #101",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #101",
        "permission denied #101",
        "bad request #101",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_101() {
    let good = format!("Bearer tok_101");
    let lower = format!("bearer tok_101");
    let bad = format!("Basic tok_101");

    assert_eq!(parse_bearer_auth(&good), Some("tok_101"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_101"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_101() {
    let expected = if 101 < 9 { 2u64.pow(101u32) } else { 256 };
    assert_eq!(reconnect_backoff(101), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_102() {
    let transient = [
        "timeout #102",
        "connection reset by peer #102",
        "broken pipe #102",
        "temporarily unavailable #102",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #102",
        "permission denied #102",
        "bad request #102",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_102() {
    let good = format!("Bearer tok_102");
    let lower = format!("bearer tok_102");
    let bad = format!("Basic tok_102");

    assert_eq!(parse_bearer_auth(&good), Some("tok_102"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_102"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_102() {
    let expected = if 102 < 9 { 2u64.pow(102u32) } else { 256 };
    assert_eq!(reconnect_backoff(102), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_103() {
    let transient = [
        "timeout #103",
        "connection reset by peer #103",
        "broken pipe #103",
        "temporarily unavailable #103",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #103",
        "permission denied #103",
        "bad request #103",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_103() {
    let good = format!("Bearer tok_103");
    let lower = format!("bearer tok_103");
    let bad = format!("Basic tok_103");

    assert_eq!(parse_bearer_auth(&good), Some("tok_103"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_103"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_103() {
    let expected = if 103 < 9 { 2u64.pow(103u32) } else { 256 };
    assert_eq!(reconnect_backoff(103), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_104() {
    let transient = [
        "timeout #104",
        "connection reset by peer #104",
        "broken pipe #104",
        "temporarily unavailable #104",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #104",
        "permission denied #104",
        "bad request #104",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_104() {
    let good = format!("Bearer tok_104");
    let lower = format!("bearer tok_104");
    let bad = format!("Basic tok_104");

    assert_eq!(parse_bearer_auth(&good), Some("tok_104"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_104"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_104() {
    let expected = if 104 < 9 { 2u64.pow(104u32) } else { 256 };
    assert_eq!(reconnect_backoff(104), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_105() {
    let transient = [
        "timeout #105",
        "connection reset by peer #105",
        "broken pipe #105",
        "temporarily unavailable #105",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #105",
        "permission denied #105",
        "bad request #105",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_105() {
    let good = format!("Bearer tok_105");
    let lower = format!("bearer tok_105");
    let bad = format!("Basic tok_105");

    assert_eq!(parse_bearer_auth(&good), Some("tok_105"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_105"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_105() {
    let expected = if 105 < 9 { 2u64.pow(105u32) } else { 256 };
    assert_eq!(reconnect_backoff(105), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_106() {
    let transient = [
        "timeout #106",
        "connection reset by peer #106",
        "broken pipe #106",
        "temporarily unavailable #106",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #106",
        "permission denied #106",
        "bad request #106",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_106() {
    let good = format!("Bearer tok_106");
    let lower = format!("bearer tok_106");
    let bad = format!("Basic tok_106");

    assert_eq!(parse_bearer_auth(&good), Some("tok_106"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_106"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_106() {
    let expected = if 106 < 9 { 2u64.pow(106u32) } else { 256 };
    assert_eq!(reconnect_backoff(106), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_107() {
    let transient = [
        "timeout #107",
        "connection reset by peer #107",
        "broken pipe #107",
        "temporarily unavailable #107",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #107",
        "permission denied #107",
        "bad request #107",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_107() {
    let good = format!("Bearer tok_107");
    let lower = format!("bearer tok_107");
    let bad = format!("Basic tok_107");

    assert_eq!(parse_bearer_auth(&good), Some("tok_107"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_107"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_107() {
    let expected = if 107 < 9 { 2u64.pow(107u32) } else { 256 };
    assert_eq!(reconnect_backoff(107), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_108() {
    let transient = [
        "timeout #108",
        "connection reset by peer #108",
        "broken pipe #108",
        "temporarily unavailable #108",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #108",
        "permission denied #108",
        "bad request #108",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_108() {
    let good = format!("Bearer tok_108");
    let lower = format!("bearer tok_108");
    let bad = format!("Basic tok_108");

    assert_eq!(parse_bearer_auth(&good), Some("tok_108"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_108"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_108() {
    let expected = if 108 < 9 { 2u64.pow(108u32) } else { 256 };
    assert_eq!(reconnect_backoff(108), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_109() {
    let transient = [
        "timeout #109",
        "connection reset by peer #109",
        "broken pipe #109",
        "temporarily unavailable #109",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #109",
        "permission denied #109",
        "bad request #109",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_109() {
    let good = format!("Bearer tok_109");
    let lower = format!("bearer tok_109");
    let bad = format!("Basic tok_109");

    assert_eq!(parse_bearer_auth(&good), Some("tok_109"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_109"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_109() {
    let expected = if 109 < 9 { 2u64.pow(109u32) } else { 256 };
    assert_eq!(reconnect_backoff(109), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_110() {
    let transient = [
        "timeout #110",
        "connection reset by peer #110",
        "broken pipe #110",
        "temporarily unavailable #110",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #110",
        "permission denied #110",
        "bad request #110",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_110() {
    let good = format!("Bearer tok_110");
    let lower = format!("bearer tok_110");
    let bad = format!("Basic tok_110");

    assert_eq!(parse_bearer_auth(&good), Some("tok_110"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_110"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_110() {
    let expected = if 110 < 9 { 2u64.pow(110u32) } else { 256 };
    assert_eq!(reconnect_backoff(110), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_111() {
    let transient = [
        "timeout #111",
        "connection reset by peer #111",
        "broken pipe #111",
        "temporarily unavailable #111",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #111",
        "permission denied #111",
        "bad request #111",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_111() {
    let good = format!("Bearer tok_111");
    let lower = format!("bearer tok_111");
    let bad = format!("Basic tok_111");

    assert_eq!(parse_bearer_auth(&good), Some("tok_111"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_111"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_111() {
    let expected = if 111 < 9 { 2u64.pow(111u32) } else { 256 };
    assert_eq!(reconnect_backoff(111), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_112() {
    let transient = [
        "timeout #112",
        "connection reset by peer #112",
        "broken pipe #112",
        "temporarily unavailable #112",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #112",
        "permission denied #112",
        "bad request #112",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_112() {
    let good = format!("Bearer tok_112");
    let lower = format!("bearer tok_112");
    let bad = format!("Basic tok_112");

    assert_eq!(parse_bearer_auth(&good), Some("tok_112"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_112"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_112() {
    let expected = if 112 < 9 { 2u64.pow(112u32) } else { 256 };
    assert_eq!(reconnect_backoff(112), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_113() {
    let transient = [
        "timeout #113",
        "connection reset by peer #113",
        "broken pipe #113",
        "temporarily unavailable #113",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #113",
        "permission denied #113",
        "bad request #113",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_113() {
    let good = format!("Bearer tok_113");
    let lower = format!("bearer tok_113");
    let bad = format!("Basic tok_113");

    assert_eq!(parse_bearer_auth(&good), Some("tok_113"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_113"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_113() {
    let expected = if 113 < 9 { 2u64.pow(113u32) } else { 256 };
    assert_eq!(reconnect_backoff(113), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_114() {
    let transient = [
        "timeout #114",
        "connection reset by peer #114",
        "broken pipe #114",
        "temporarily unavailable #114",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #114",
        "permission denied #114",
        "bad request #114",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_114() {
    let good = format!("Bearer tok_114");
    let lower = format!("bearer tok_114");
    let bad = format!("Basic tok_114");

    assert_eq!(parse_bearer_auth(&good), Some("tok_114"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_114"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_114() {
    let expected = if 114 < 9 { 2u64.pow(114u32) } else { 256 };
    assert_eq!(reconnect_backoff(114), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_115() {
    let transient = [
        "timeout #115",
        "connection reset by peer #115",
        "broken pipe #115",
        "temporarily unavailable #115",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #115",
        "permission denied #115",
        "bad request #115",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_115() {
    let good = format!("Bearer tok_115");
    let lower = format!("bearer tok_115");
    let bad = format!("Basic tok_115");

    assert_eq!(parse_bearer_auth(&good), Some("tok_115"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_115"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_115() {
    let expected = if 115 < 9 { 2u64.pow(115u32) } else { 256 };
    assert_eq!(reconnect_backoff(115), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_116() {
    let transient = [
        "timeout #116",
        "connection reset by peer #116",
        "broken pipe #116",
        "temporarily unavailable #116",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #116",
        "permission denied #116",
        "bad request #116",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_116() {
    let good = format!("Bearer tok_116");
    let lower = format!("bearer tok_116");
    let bad = format!("Basic tok_116");

    assert_eq!(parse_bearer_auth(&good), Some("tok_116"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_116"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_116() {
    let expected = if 116 < 9 { 2u64.pow(116u32) } else { 256 };
    assert_eq!(reconnect_backoff(116), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_117() {
    let transient = [
        "timeout #117",
        "connection reset by peer #117",
        "broken pipe #117",
        "temporarily unavailable #117",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #117",
        "permission denied #117",
        "bad request #117",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_117() {
    let good = format!("Bearer tok_117");
    let lower = format!("bearer tok_117");
    let bad = format!("Basic tok_117");

    assert_eq!(parse_bearer_auth(&good), Some("tok_117"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_117"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_117() {
    let expected = if 117 < 9 { 2u64.pow(117u32) } else { 256 };
    assert_eq!(reconnect_backoff(117), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_118() {
    let transient = [
        "timeout #118",
        "connection reset by peer #118",
        "broken pipe #118",
        "temporarily unavailable #118",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #118",
        "permission denied #118",
        "bad request #118",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_118() {
    let good = format!("Bearer tok_118");
    let lower = format!("bearer tok_118");
    let bad = format!("Basic tok_118");

    assert_eq!(parse_bearer_auth(&good), Some("tok_118"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_118"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_118() {
    let expected = if 118 < 9 { 2u64.pow(118u32) } else { 256 };
    assert_eq!(reconnect_backoff(118), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_119() {
    let transient = [
        "timeout #119",
        "connection reset by peer #119",
        "broken pipe #119",
        "temporarily unavailable #119",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #119",
        "permission denied #119",
        "bad request #119",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_119() {
    let good = format!("Bearer tok_119");
    let lower = format!("bearer tok_119");
    let bad = format!("Basic tok_119");

    assert_eq!(parse_bearer_auth(&good), Some("tok_119"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_119"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_119() {
    let expected = if 119 < 9 { 2u64.pow(119u32) } else { 256 };
    assert_eq!(reconnect_backoff(119), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_120() {
    let transient = [
        "timeout #120",
        "connection reset by peer #120",
        "broken pipe #120",
        "temporarily unavailable #120",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #120",
        "permission denied #120",
        "bad request #120",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_120() {
    let good = format!("Bearer tok_120");
    let lower = format!("bearer tok_120");
    let bad = format!("Basic tok_120");

    assert_eq!(parse_bearer_auth(&good), Some("tok_120"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_120"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_120() {
    let expected = if 120 < 9 { 2u64.pow(120u32) } else { 256 };
    assert_eq!(reconnect_backoff(120), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_121() {
    let transient = [
        "timeout #121",
        "connection reset by peer #121",
        "broken pipe #121",
        "temporarily unavailable #121",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #121",
        "permission denied #121",
        "bad request #121",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_121() {
    let good = format!("Bearer tok_121");
    let lower = format!("bearer tok_121");
    let bad = format!("Basic tok_121");

    assert_eq!(parse_bearer_auth(&good), Some("tok_121"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_121"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_121() {
    let expected = if 121 < 9 { 2u64.pow(121u32) } else { 256 };
    assert_eq!(reconnect_backoff(121), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_122() {
    let transient = [
        "timeout #122",
        "connection reset by peer #122",
        "broken pipe #122",
        "temporarily unavailable #122",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #122",
        "permission denied #122",
        "bad request #122",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_122() {
    let good = format!("Bearer tok_122");
    let lower = format!("bearer tok_122");
    let bad = format!("Basic tok_122");

    assert_eq!(parse_bearer_auth(&good), Some("tok_122"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_122"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_122() {
    let expected = if 122 < 9 { 2u64.pow(122u32) } else { 256 };
    assert_eq!(reconnect_backoff(122), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_123() {
    let transient = [
        "timeout #123",
        "connection reset by peer #123",
        "broken pipe #123",
        "temporarily unavailable #123",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #123",
        "permission denied #123",
        "bad request #123",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_123() {
    let good = format!("Bearer tok_123");
    let lower = format!("bearer tok_123");
    let bad = format!("Basic tok_123");

    assert_eq!(parse_bearer_auth(&good), Some("tok_123"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_123"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_123() {
    let expected = if 123 < 9 { 2u64.pow(123u32) } else { 256 };
    assert_eq!(reconnect_backoff(123), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_124() {
    let transient = [
        "timeout #124",
        "connection reset by peer #124",
        "broken pipe #124",
        "temporarily unavailable #124",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #124",
        "permission denied #124",
        "bad request #124",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_124() {
    let good = format!("Bearer tok_124");
    let lower = format!("bearer tok_124");
    let bad = format!("Basic tok_124");

    assert_eq!(parse_bearer_auth(&good), Some("tok_124"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_124"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_124() {
    let expected = if 124 < 9 { 2u64.pow(124u32) } else { 256 };
    assert_eq!(reconnect_backoff(124), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_125() {
    let transient = [
        "timeout #125",
        "connection reset by peer #125",
        "broken pipe #125",
        "temporarily unavailable #125",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #125",
        "permission denied #125",
        "bad request #125",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_125() {
    let good = format!("Bearer tok_125");
    let lower = format!("bearer tok_125");
    let bad = format!("Basic tok_125");

    assert_eq!(parse_bearer_auth(&good), Some("tok_125"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_125"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_125() {
    let expected = if 125 < 9 { 2u64.pow(125u32) } else { 256 };
    assert_eq!(reconnect_backoff(125), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_126() {
    let transient = [
        "timeout #126",
        "connection reset by peer #126",
        "broken pipe #126",
        "temporarily unavailable #126",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #126",
        "permission denied #126",
        "bad request #126",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_126() {
    let good = format!("Bearer tok_126");
    let lower = format!("bearer tok_126");
    let bad = format!("Basic tok_126");

    assert_eq!(parse_bearer_auth(&good), Some("tok_126"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_126"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_126() {
    let expected = if 126 < 9 { 2u64.pow(126u32) } else { 256 };
    assert_eq!(reconnect_backoff(126), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_127() {
    let transient = [
        "timeout #127",
        "connection reset by peer #127",
        "broken pipe #127",
        "temporarily unavailable #127",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #127",
        "permission denied #127",
        "bad request #127",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_127() {
    let good = format!("Bearer tok_127");
    let lower = format!("bearer tok_127");
    let bad = format!("Basic tok_127");

    assert_eq!(parse_bearer_auth(&good), Some("tok_127"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_127"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_127() {
    let expected = if 127 < 9 { 2u64.pow(127u32) } else { 256 };
    assert_eq!(reconnect_backoff(127), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_128() {
    let transient = [
        "timeout #128",
        "connection reset by peer #128",
        "broken pipe #128",
        "temporarily unavailable #128",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #128",
        "permission denied #128",
        "bad request #128",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_128() {
    let good = format!("Bearer tok_128");
    let lower = format!("bearer tok_128");
    let bad = format!("Basic tok_128");

    assert_eq!(parse_bearer_auth(&good), Some("tok_128"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_128"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_128() {
    let expected = if 128 < 9 { 2u64.pow(128u32) } else { 256 };
    assert_eq!(reconnect_backoff(128), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_129() {
    let transient = [
        "timeout #129",
        "connection reset by peer #129",
        "broken pipe #129",
        "temporarily unavailable #129",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #129",
        "permission denied #129",
        "bad request #129",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_129() {
    let good = format!("Bearer tok_129");
    let lower = format!("bearer tok_129");
    let bad = format!("Basic tok_129");

    assert_eq!(parse_bearer_auth(&good), Some("tok_129"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_129"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_129() {
    let expected = if 129 < 9 { 2u64.pow(129u32) } else { 256 };
    assert_eq!(reconnect_backoff(129), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_130() {
    let transient = [
        "timeout #130",
        "connection reset by peer #130",
        "broken pipe #130",
        "temporarily unavailable #130",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #130",
        "permission denied #130",
        "bad request #130",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_130() {
    let good = format!("Bearer tok_130");
    let lower = format!("bearer tok_130");
    let bad = format!("Basic tok_130");

    assert_eq!(parse_bearer_auth(&good), Some("tok_130"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_130"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_130() {
    let expected = if 130 < 9 { 2u64.pow(130u32) } else { 256 };
    assert_eq!(reconnect_backoff(130), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_131() {
    let transient = [
        "timeout #131",
        "connection reset by peer #131",
        "broken pipe #131",
        "temporarily unavailable #131",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #131",
        "permission denied #131",
        "bad request #131",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_131() {
    let good = format!("Bearer tok_131");
    let lower = format!("bearer tok_131");
    let bad = format!("Basic tok_131");

    assert_eq!(parse_bearer_auth(&good), Some("tok_131"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_131"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_131() {
    let expected = if 131 < 9 { 2u64.pow(131u32) } else { 256 };
    assert_eq!(reconnect_backoff(131), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_132() {
    let transient = [
        "timeout #132",
        "connection reset by peer #132",
        "broken pipe #132",
        "temporarily unavailable #132",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #132",
        "permission denied #132",
        "bad request #132",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_132() {
    let good = format!("Bearer tok_132");
    let lower = format!("bearer tok_132");
    let bad = format!("Basic tok_132");

    assert_eq!(parse_bearer_auth(&good), Some("tok_132"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_132"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_132() {
    let expected = if 132 < 9 { 2u64.pow(132u32) } else { 256 };
    assert_eq!(reconnect_backoff(132), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_133() {
    let transient = [
        "timeout #133",
        "connection reset by peer #133",
        "broken pipe #133",
        "temporarily unavailable #133",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #133",
        "permission denied #133",
        "bad request #133",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_133() {
    let good = format!("Bearer tok_133");
    let lower = format!("bearer tok_133");
    let bad = format!("Basic tok_133");

    assert_eq!(parse_bearer_auth(&good), Some("tok_133"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_133"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_133() {
    let expected = if 133 < 9 { 2u64.pow(133u32) } else { 256 };
    assert_eq!(reconnect_backoff(133), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_134() {
    let transient = [
        "timeout #134",
        "connection reset by peer #134",
        "broken pipe #134",
        "temporarily unavailable #134",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #134",
        "permission denied #134",
        "bad request #134",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_134() {
    let good = format!("Bearer tok_134");
    let lower = format!("bearer tok_134");
    let bad = format!("Basic tok_134");

    assert_eq!(parse_bearer_auth(&good), Some("tok_134"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_134"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_134() {
    let expected = if 134 < 9 { 2u64.pow(134u32) } else { 256 };
    assert_eq!(reconnect_backoff(134), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_135() {
    let transient = [
        "timeout #135",
        "connection reset by peer #135",
        "broken pipe #135",
        "temporarily unavailable #135",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #135",
        "permission denied #135",
        "bad request #135",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_135() {
    let good = format!("Bearer tok_135");
    let lower = format!("bearer tok_135");
    let bad = format!("Basic tok_135");

    assert_eq!(parse_bearer_auth(&good), Some("tok_135"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_135"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_135() {
    let expected = if 135 < 9 { 2u64.pow(135u32) } else { 256 };
    assert_eq!(reconnect_backoff(135), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_136() {
    let transient = [
        "timeout #136",
        "connection reset by peer #136",
        "broken pipe #136",
        "temporarily unavailable #136",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #136",
        "permission denied #136",
        "bad request #136",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_136() {
    let good = format!("Bearer tok_136");
    let lower = format!("bearer tok_136");
    let bad = format!("Basic tok_136");

    assert_eq!(parse_bearer_auth(&good), Some("tok_136"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_136"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_136() {
    let expected = if 136 < 9 { 2u64.pow(136u32) } else { 256 };
    assert_eq!(reconnect_backoff(136), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_137() {
    let transient = [
        "timeout #137",
        "connection reset by peer #137",
        "broken pipe #137",
        "temporarily unavailable #137",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #137",
        "permission denied #137",
        "bad request #137",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_137() {
    let good = format!("Bearer tok_137");
    let lower = format!("bearer tok_137");
    let bad = format!("Basic tok_137");

    assert_eq!(parse_bearer_auth(&good), Some("tok_137"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_137"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_137() {
    let expected = if 137 < 9 { 2u64.pow(137u32) } else { 256 };
    assert_eq!(reconnect_backoff(137), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_138() {
    let transient = [
        "timeout #138",
        "connection reset by peer #138",
        "broken pipe #138",
        "temporarily unavailable #138",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #138",
        "permission denied #138",
        "bad request #138",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_138() {
    let good = format!("Bearer tok_138");
    let lower = format!("bearer tok_138");
    let bad = format!("Basic tok_138");

    assert_eq!(parse_bearer_auth(&good), Some("tok_138"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_138"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_138() {
    let expected = if 138 < 9 { 2u64.pow(138u32) } else { 256 };
    assert_eq!(reconnect_backoff(138), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_139() {
    let transient = [
        "timeout #139",
        "connection reset by peer #139",
        "broken pipe #139",
        "temporarily unavailable #139",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #139",
        "permission denied #139",
        "bad request #139",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_139() {
    let good = format!("Bearer tok_139");
    let lower = format!("bearer tok_139");
    let bad = format!("Basic tok_139");

    assert_eq!(parse_bearer_auth(&good), Some("tok_139"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_139"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_139() {
    let expected = if 139 < 9 { 2u64.pow(139u32) } else { 256 };
    assert_eq!(reconnect_backoff(139), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_140() {
    let transient = [
        "timeout #140",
        "connection reset by peer #140",
        "broken pipe #140",
        "temporarily unavailable #140",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #140",
        "permission denied #140",
        "bad request #140",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_140() {
    let good = format!("Bearer tok_140");
    let lower = format!("bearer tok_140");
    let bad = format!("Basic tok_140");

    assert_eq!(parse_bearer_auth(&good), Some("tok_140"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_140"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_140() {
    let expected = if 140 < 9 { 2u64.pow(140u32) } else { 256 };
    assert_eq!(reconnect_backoff(140), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_141() {
    let transient = [
        "timeout #141",
        "connection reset by peer #141",
        "broken pipe #141",
        "temporarily unavailable #141",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #141",
        "permission denied #141",
        "bad request #141",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_141() {
    let good = format!("Bearer tok_141");
    let lower = format!("bearer tok_141");
    let bad = format!("Basic tok_141");

    assert_eq!(parse_bearer_auth(&good), Some("tok_141"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_141"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_141() {
    let expected = if 141 < 9 { 2u64.pow(141u32) } else { 256 };
    assert_eq!(reconnect_backoff(141), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_142() {
    let transient = [
        "timeout #142",
        "connection reset by peer #142",
        "broken pipe #142",
        "temporarily unavailable #142",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #142",
        "permission denied #142",
        "bad request #142",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_142() {
    let good = format!("Bearer tok_142");
    let lower = format!("bearer tok_142");
    let bad = format!("Basic tok_142");

    assert_eq!(parse_bearer_auth(&good), Some("tok_142"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_142"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_142() {
    let expected = if 142 < 9 { 2u64.pow(142u32) } else { 256 };
    assert_eq!(reconnect_backoff(142), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_143() {
    let transient = [
        "timeout #143",
        "connection reset by peer #143",
        "broken pipe #143",
        "temporarily unavailable #143",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #143",
        "permission denied #143",
        "bad request #143",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_143() {
    let good = format!("Bearer tok_143");
    let lower = format!("bearer tok_143");
    let bad = format!("Basic tok_143");

    assert_eq!(parse_bearer_auth(&good), Some("tok_143"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_143"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_143() {
    let expected = if 143 < 9 { 2u64.pow(143u32) } else { 256 };
    assert_eq!(reconnect_backoff(143), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_144() {
    let transient = [
        "timeout #144",
        "connection reset by peer #144",
        "broken pipe #144",
        "temporarily unavailable #144",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #144",
        "permission denied #144",
        "bad request #144",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_144() {
    let good = format!("Bearer tok_144");
    let lower = format!("bearer tok_144");
    let bad = format!("Basic tok_144");

    assert_eq!(parse_bearer_auth(&good), Some("tok_144"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_144"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_144() {
    let expected = if 144 < 9 { 2u64.pow(144u32) } else { 256 };
    assert_eq!(reconnect_backoff(144), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_145() {
    let transient = [
        "timeout #145",
        "connection reset by peer #145",
        "broken pipe #145",
        "temporarily unavailable #145",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #145",
        "permission denied #145",
        "bad request #145",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_145() {
    let good = format!("Bearer tok_145");
    let lower = format!("bearer tok_145");
    let bad = format!("Basic tok_145");

    assert_eq!(parse_bearer_auth(&good), Some("tok_145"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_145"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_145() {
    let expected = if 145 < 9 { 2u64.pow(145u32) } else { 256 };
    assert_eq!(reconnect_backoff(145), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_146() {
    let transient = [
        "timeout #146",
        "connection reset by peer #146",
        "broken pipe #146",
        "temporarily unavailable #146",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #146",
        "permission denied #146",
        "bad request #146",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_146() {
    let good = format!("Bearer tok_146");
    let lower = format!("bearer tok_146");
    let bad = format!("Basic tok_146");

    assert_eq!(parse_bearer_auth(&good), Some("tok_146"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_146"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_146() {
    let expected = if 146 < 9 { 2u64.pow(146u32) } else { 256 };
    assert_eq!(reconnect_backoff(146), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_147() {
    let transient = [
        "timeout #147",
        "connection reset by peer #147",
        "broken pipe #147",
        "temporarily unavailable #147",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #147",
        "permission denied #147",
        "bad request #147",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_147() {
    let good = format!("Bearer tok_147");
    let lower = format!("bearer tok_147");
    let bad = format!("Basic tok_147");

    assert_eq!(parse_bearer_auth(&good), Some("tok_147"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_147"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_147() {
    let expected = if 147 < 9 { 2u64.pow(147u32) } else { 256 };
    assert_eq!(reconnect_backoff(147), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_148() {
    let transient = [
        "timeout #148",
        "connection reset by peer #148",
        "broken pipe #148",
        "temporarily unavailable #148",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #148",
        "permission denied #148",
        "bad request #148",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_148() {
    let good = format!("Bearer tok_148");
    let lower = format!("bearer tok_148");
    let bad = format!("Basic tok_148");

    assert_eq!(parse_bearer_auth(&good), Some("tok_148"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_148"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_148() {
    let expected = if 148 < 9 { 2u64.pow(148u32) } else { 256 };
    assert_eq!(reconnect_backoff(148), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_149() {
    let transient = [
        "timeout #149",
        "connection reset by peer #149",
        "broken pipe #149",
        "temporarily unavailable #149",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #149",
        "permission denied #149",
        "bad request #149",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_149() {
    let good = format!("Bearer tok_149");
    let lower = format!("bearer tok_149");
    let bad = format!("Basic tok_149");

    assert_eq!(parse_bearer_auth(&good), Some("tok_149"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_149"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_149() {
    let expected = if 149 < 9 { 2u64.pow(149u32) } else { 256 };
    assert_eq!(reconnect_backoff(149), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_150() {
    let transient = [
        "timeout #150",
        "connection reset by peer #150",
        "broken pipe #150",
        "temporarily unavailable #150",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #150",
        "permission denied #150",
        "bad request #150",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_150() {
    let good = format!("Bearer tok_150");
    let lower = format!("bearer tok_150");
    let bad = format!("Basic tok_150");

    assert_eq!(parse_bearer_auth(&good), Some("tok_150"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_150"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_150() {
    let expected = if 150 < 9 { 2u64.pow(150u32) } else { 256 };
    assert_eq!(reconnect_backoff(150), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_151() {
    let transient = [
        "timeout #151",
        "connection reset by peer #151",
        "broken pipe #151",
        "temporarily unavailable #151",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #151",
        "permission denied #151",
        "bad request #151",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_151() {
    let good = format!("Bearer tok_151");
    let lower = format!("bearer tok_151");
    let bad = format!("Basic tok_151");

    assert_eq!(parse_bearer_auth(&good), Some("tok_151"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_151"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_151() {
    let expected = if 151 < 9 { 2u64.pow(151u32) } else { 256 };
    assert_eq!(reconnect_backoff(151), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_152() {
    let transient = [
        "timeout #152",
        "connection reset by peer #152",
        "broken pipe #152",
        "temporarily unavailable #152",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #152",
        "permission denied #152",
        "bad request #152",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_152() {
    let good = format!("Bearer tok_152");
    let lower = format!("bearer tok_152");
    let bad = format!("Basic tok_152");

    assert_eq!(parse_bearer_auth(&good), Some("tok_152"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_152"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_152() {
    let expected = if 152 < 9 { 2u64.pow(152u32) } else { 256 };
    assert_eq!(reconnect_backoff(152), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_153() {
    let transient = [
        "timeout #153",
        "connection reset by peer #153",
        "broken pipe #153",
        "temporarily unavailable #153",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #153",
        "permission denied #153",
        "bad request #153",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_153() {
    let good = format!("Bearer tok_153");
    let lower = format!("bearer tok_153");
    let bad = format!("Basic tok_153");

    assert_eq!(parse_bearer_auth(&good), Some("tok_153"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_153"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_153() {
    let expected = if 153 < 9 { 2u64.pow(153u32) } else { 256 };
    assert_eq!(reconnect_backoff(153), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_154() {
    let transient = [
        "timeout #154",
        "connection reset by peer #154",
        "broken pipe #154",
        "temporarily unavailable #154",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #154",
        "permission denied #154",
        "bad request #154",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_154() {
    let good = format!("Bearer tok_154");
    let lower = format!("bearer tok_154");
    let bad = format!("Basic tok_154");

    assert_eq!(parse_bearer_auth(&good), Some("tok_154"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_154"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_154() {
    let expected = if 154 < 9 { 2u64.pow(154u32) } else { 256 };
    assert_eq!(reconnect_backoff(154), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_155() {
    let transient = [
        "timeout #155",
        "connection reset by peer #155",
        "broken pipe #155",
        "temporarily unavailable #155",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #155",
        "permission denied #155",
        "bad request #155",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_155() {
    let good = format!("Bearer tok_155");
    let lower = format!("bearer tok_155");
    let bad = format!("Basic tok_155");

    assert_eq!(parse_bearer_auth(&good), Some("tok_155"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_155"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_155() {
    let expected = if 155 < 9 { 2u64.pow(155u32) } else { 256 };
    assert_eq!(reconnect_backoff(155), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_156() {
    let transient = [
        "timeout #156",
        "connection reset by peer #156",
        "broken pipe #156",
        "temporarily unavailable #156",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #156",
        "permission denied #156",
        "bad request #156",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_156() {
    let good = format!("Bearer tok_156");
    let lower = format!("bearer tok_156");
    let bad = format!("Basic tok_156");

    assert_eq!(parse_bearer_auth(&good), Some("tok_156"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_156"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_156() {
    let expected = if 156 < 9 { 2u64.pow(156u32) } else { 256 };
    assert_eq!(reconnect_backoff(156), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_157() {
    let transient = [
        "timeout #157",
        "connection reset by peer #157",
        "broken pipe #157",
        "temporarily unavailable #157",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #157",
        "permission denied #157",
        "bad request #157",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_157() {
    let good = format!("Bearer tok_157");
    let lower = format!("bearer tok_157");
    let bad = format!("Basic tok_157");

    assert_eq!(parse_bearer_auth(&good), Some("tok_157"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_157"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_157() {
    let expected = if 157 < 9 { 2u64.pow(157u32) } else { 256 };
    assert_eq!(reconnect_backoff(157), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_158() {
    let transient = [
        "timeout #158",
        "connection reset by peer #158",
        "broken pipe #158",
        "temporarily unavailable #158",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #158",
        "permission denied #158",
        "bad request #158",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_158() {
    let good = format!("Bearer tok_158");
    let lower = format!("bearer tok_158");
    let bad = format!("Basic tok_158");

    assert_eq!(parse_bearer_auth(&good), Some("tok_158"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_158"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_158() {
    let expected = if 158 < 9 { 2u64.pow(158u32) } else { 256 };
    assert_eq!(reconnect_backoff(158), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_159() {
    let transient = [
        "timeout #159",
        "connection reset by peer #159",
        "broken pipe #159",
        "temporarily unavailable #159",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #159",
        "permission denied #159",
        "bad request #159",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_159() {
    let good = format!("Bearer tok_159");
    let lower = format!("bearer tok_159");
    let bad = format!("Basic tok_159");

    assert_eq!(parse_bearer_auth(&good), Some("tok_159"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_159"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_159() {
    let expected = if 159 < 9 { 2u64.pow(159u32) } else { 256 };
    assert_eq!(reconnect_backoff(159), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_160() {
    let transient = [
        "timeout #160",
        "connection reset by peer #160",
        "broken pipe #160",
        "temporarily unavailable #160",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #160",
        "permission denied #160",
        "bad request #160",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_160() {
    let good = format!("Bearer tok_160");
    let lower = format!("bearer tok_160");
    let bad = format!("Basic tok_160");

    assert_eq!(parse_bearer_auth(&good), Some("tok_160"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_160"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_160() {
    let expected = if 160 < 9 { 2u64.pow(160u32) } else { 256 };
    assert_eq!(reconnect_backoff(160), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_161() {
    let transient = [
        "timeout #161",
        "connection reset by peer #161",
        "broken pipe #161",
        "temporarily unavailable #161",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #161",
        "permission denied #161",
        "bad request #161",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_161() {
    let good = format!("Bearer tok_161");
    let lower = format!("bearer tok_161");
    let bad = format!("Basic tok_161");

    assert_eq!(parse_bearer_auth(&good), Some("tok_161"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_161"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_161() {
    let expected = if 161 < 9 { 2u64.pow(161u32) } else { 256 };
    assert_eq!(reconnect_backoff(161), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_162() {
    let transient = [
        "timeout #162",
        "connection reset by peer #162",
        "broken pipe #162",
        "temporarily unavailable #162",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #162",
        "permission denied #162",
        "bad request #162",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_162() {
    let good = format!("Bearer tok_162");
    let lower = format!("bearer tok_162");
    let bad = format!("Basic tok_162");

    assert_eq!(parse_bearer_auth(&good), Some("tok_162"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_162"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_162() {
    let expected = if 162 < 9 { 2u64.pow(162u32) } else { 256 };
    assert_eq!(reconnect_backoff(162), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_163() {
    let transient = [
        "timeout #163",
        "connection reset by peer #163",
        "broken pipe #163",
        "temporarily unavailable #163",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #163",
        "permission denied #163",
        "bad request #163",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_163() {
    let good = format!("Bearer tok_163");
    let lower = format!("bearer tok_163");
    let bad = format!("Basic tok_163");

    assert_eq!(parse_bearer_auth(&good), Some("tok_163"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_163"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_163() {
    let expected = if 163 < 9 { 2u64.pow(163u32) } else { 256 };
    assert_eq!(reconnect_backoff(163), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_164() {
    let transient = [
        "timeout #164",
        "connection reset by peer #164",
        "broken pipe #164",
        "temporarily unavailable #164",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #164",
        "permission denied #164",
        "bad request #164",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_164() {
    let good = format!("Bearer tok_164");
    let lower = format!("bearer tok_164");
    let bad = format!("Basic tok_164");

    assert_eq!(parse_bearer_auth(&good), Some("tok_164"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_164"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_164() {
    let expected = if 164 < 9 { 2u64.pow(164u32) } else { 256 };
    assert_eq!(reconnect_backoff(164), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_165() {
    let transient = [
        "timeout #165",
        "connection reset by peer #165",
        "broken pipe #165",
        "temporarily unavailable #165",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #165",
        "permission denied #165",
        "bad request #165",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_165() {
    let good = format!("Bearer tok_165");
    let lower = format!("bearer tok_165");
    let bad = format!("Basic tok_165");

    assert_eq!(parse_bearer_auth(&good), Some("tok_165"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_165"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_165() {
    let expected = if 165 < 9 { 2u64.pow(165u32) } else { 256 };
    assert_eq!(reconnect_backoff(165), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_166() {
    let transient = [
        "timeout #166",
        "connection reset by peer #166",
        "broken pipe #166",
        "temporarily unavailable #166",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #166",
        "permission denied #166",
        "bad request #166",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_166() {
    let good = format!("Bearer tok_166");
    let lower = format!("bearer tok_166");
    let bad = format!("Basic tok_166");

    assert_eq!(parse_bearer_auth(&good), Some("tok_166"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_166"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_166() {
    let expected = if 166 < 9 { 2u64.pow(166u32) } else { 256 };
    assert_eq!(reconnect_backoff(166), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_167() {
    let transient = [
        "timeout #167",
        "connection reset by peer #167",
        "broken pipe #167",
        "temporarily unavailable #167",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #167",
        "permission denied #167",
        "bad request #167",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_167() {
    let good = format!("Bearer tok_167");
    let lower = format!("bearer tok_167");
    let bad = format!("Basic tok_167");

    assert_eq!(parse_bearer_auth(&good), Some("tok_167"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_167"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_167() {
    let expected = if 167 < 9 { 2u64.pow(167u32) } else { 256 };
    assert_eq!(reconnect_backoff(167), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_168() {
    let transient = [
        "timeout #168",
        "connection reset by peer #168",
        "broken pipe #168",
        "temporarily unavailable #168",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #168",
        "permission denied #168",
        "bad request #168",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_168() {
    let good = format!("Bearer tok_168");
    let lower = format!("bearer tok_168");
    let bad = format!("Basic tok_168");

    assert_eq!(parse_bearer_auth(&good), Some("tok_168"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_168"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_168() {
    let expected = if 168 < 9 { 2u64.pow(168u32) } else { 256 };
    assert_eq!(reconnect_backoff(168), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_169() {
    let transient = [
        "timeout #169",
        "connection reset by peer #169",
        "broken pipe #169",
        "temporarily unavailable #169",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #169",
        "permission denied #169",
        "bad request #169",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_169() {
    let good = format!("Bearer tok_169");
    let lower = format!("bearer tok_169");
    let bad = format!("Basic tok_169");

    assert_eq!(parse_bearer_auth(&good), Some("tok_169"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_169"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_169() {
    let expected = if 169 < 9 { 2u64.pow(169u32) } else { 256 };
    assert_eq!(reconnect_backoff(169), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_170() {
    let transient = [
        "timeout #170",
        "connection reset by peer #170",
        "broken pipe #170",
        "temporarily unavailable #170",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #170",
        "permission denied #170",
        "bad request #170",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_170() {
    let good = format!("Bearer tok_170");
    let lower = format!("bearer tok_170");
    let bad = format!("Basic tok_170");

    assert_eq!(parse_bearer_auth(&good), Some("tok_170"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_170"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_170() {
    let expected = if 170 < 9 { 2u64.pow(170u32) } else { 256 };
    assert_eq!(reconnect_backoff(170), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_171() {
    let transient = [
        "timeout #171",
        "connection reset by peer #171",
        "broken pipe #171",
        "temporarily unavailable #171",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #171",
        "permission denied #171",
        "bad request #171",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_171() {
    let good = format!("Bearer tok_171");
    let lower = format!("bearer tok_171");
    let bad = format!("Basic tok_171");

    assert_eq!(parse_bearer_auth(&good), Some("tok_171"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_171"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_171() {
    let expected = if 171 < 9 { 2u64.pow(171u32) } else { 256 };
    assert_eq!(reconnect_backoff(171), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_172() {
    let transient = [
        "timeout #172",
        "connection reset by peer #172",
        "broken pipe #172",
        "temporarily unavailable #172",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #172",
        "permission denied #172",
        "bad request #172",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_172() {
    let good = format!("Bearer tok_172");
    let lower = format!("bearer tok_172");
    let bad = format!("Basic tok_172");

    assert_eq!(parse_bearer_auth(&good), Some("tok_172"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_172"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_172() {
    let expected = if 172 < 9 { 2u64.pow(172u32) } else { 256 };
    assert_eq!(reconnect_backoff(172), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_173() {
    let transient = [
        "timeout #173",
        "connection reset by peer #173",
        "broken pipe #173",
        "temporarily unavailable #173",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #173",
        "permission denied #173",
        "bad request #173",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_173() {
    let good = format!("Bearer tok_173");
    let lower = format!("bearer tok_173");
    let bad = format!("Basic tok_173");

    assert_eq!(parse_bearer_auth(&good), Some("tok_173"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_173"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_173() {
    let expected = if 173 < 9 { 2u64.pow(173u32) } else { 256 };
    assert_eq!(reconnect_backoff(173), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_174() {
    let transient = [
        "timeout #174",
        "connection reset by peer #174",
        "broken pipe #174",
        "temporarily unavailable #174",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #174",
        "permission denied #174",
        "bad request #174",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_174() {
    let good = format!("Bearer tok_174");
    let lower = format!("bearer tok_174");
    let bad = format!("Basic tok_174");

    assert_eq!(parse_bearer_auth(&good), Some("tok_174"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_174"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_174() {
    let expected = if 174 < 9 { 2u64.pow(174u32) } else { 256 };
    assert_eq!(reconnect_backoff(174), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_175() {
    let transient = [
        "timeout #175",
        "connection reset by peer #175",
        "broken pipe #175",
        "temporarily unavailable #175",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #175",
        "permission denied #175",
        "bad request #175",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_175() {
    let good = format!("Bearer tok_175");
    let lower = format!("bearer tok_175");
    let bad = format!("Basic tok_175");

    assert_eq!(parse_bearer_auth(&good), Some("tok_175"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_175"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_175() {
    let expected = if 175 < 9 { 2u64.pow(175u32) } else { 256 };
    assert_eq!(reconnect_backoff(175), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_176() {
    let transient = [
        "timeout #176",
        "connection reset by peer #176",
        "broken pipe #176",
        "temporarily unavailable #176",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #176",
        "permission denied #176",
        "bad request #176",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_176() {
    let good = format!("Bearer tok_176");
    let lower = format!("bearer tok_176");
    let bad = format!("Basic tok_176");

    assert_eq!(parse_bearer_auth(&good), Some("tok_176"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_176"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_176() {
    let expected = if 176 < 9 { 2u64.pow(176u32) } else { 256 };
    assert_eq!(reconnect_backoff(176), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_177() {
    let transient = [
        "timeout #177",
        "connection reset by peer #177",
        "broken pipe #177",
        "temporarily unavailable #177",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #177",
        "permission denied #177",
        "bad request #177",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_177() {
    let good = format!("Bearer tok_177");
    let lower = format!("bearer tok_177");
    let bad = format!("Basic tok_177");

    assert_eq!(parse_bearer_auth(&good), Some("tok_177"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_177"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_177() {
    let expected = if 177 < 9 { 2u64.pow(177u32) } else { 256 };
    assert_eq!(reconnect_backoff(177), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_178() {
    let transient = [
        "timeout #178",
        "connection reset by peer #178",
        "broken pipe #178",
        "temporarily unavailable #178",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #178",
        "permission denied #178",
        "bad request #178",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_178() {
    let good = format!("Bearer tok_178");
    let lower = format!("bearer tok_178");
    let bad = format!("Basic tok_178");

    assert_eq!(parse_bearer_auth(&good), Some("tok_178"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_178"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_178() {
    let expected = if 178 < 9 { 2u64.pow(178u32) } else { 256 };
    assert_eq!(reconnect_backoff(178), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_179() {
    let transient = [
        "timeout #179",
        "connection reset by peer #179",
        "broken pipe #179",
        "temporarily unavailable #179",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #179",
        "permission denied #179",
        "bad request #179",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_179() {
    let good = format!("Bearer tok_179");
    let lower = format!("bearer tok_179");
    let bad = format!("Basic tok_179");

    assert_eq!(parse_bearer_auth(&good), Some("tok_179"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_179"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_179() {
    let expected = if 179 < 9 { 2u64.pow(179u32) } else { 256 };
    assert_eq!(reconnect_backoff(179), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_180() {
    let transient = [
        "timeout #180",
        "connection reset by peer #180",
        "broken pipe #180",
        "temporarily unavailable #180",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #180",
        "permission denied #180",
        "bad request #180",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_180() {
    let good = format!("Bearer tok_180");
    let lower = format!("bearer tok_180");
    let bad = format!("Basic tok_180");

    assert_eq!(parse_bearer_auth(&good), Some("tok_180"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_180"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_180() {
    let expected = if 180 < 9 { 2u64.pow(180u32) } else { 256 };
    assert_eq!(reconnect_backoff(180), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_181() {
    let transient = [
        "timeout #181",
        "connection reset by peer #181",
        "broken pipe #181",
        "temporarily unavailable #181",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #181",
        "permission denied #181",
        "bad request #181",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_181() {
    let good = format!("Bearer tok_181");
    let lower = format!("bearer tok_181");
    let bad = format!("Basic tok_181");

    assert_eq!(parse_bearer_auth(&good), Some("tok_181"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_181"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_181() {
    let expected = if 181 < 9 { 2u64.pow(181u32) } else { 256 };
    assert_eq!(reconnect_backoff(181), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_182() {
    let transient = [
        "timeout #182",
        "connection reset by peer #182",
        "broken pipe #182",
        "temporarily unavailable #182",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #182",
        "permission denied #182",
        "bad request #182",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_182() {
    let good = format!("Bearer tok_182");
    let lower = format!("bearer tok_182");
    let bad = format!("Basic tok_182");

    assert_eq!(parse_bearer_auth(&good), Some("tok_182"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_182"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_182() {
    let expected = if 182 < 9 { 2u64.pow(182u32) } else { 256 };
    assert_eq!(reconnect_backoff(182), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_183() {
    let transient = [
        "timeout #183",
        "connection reset by peer #183",
        "broken pipe #183",
        "temporarily unavailable #183",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #183",
        "permission denied #183",
        "bad request #183",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_183() {
    let good = format!("Bearer tok_183");
    let lower = format!("bearer tok_183");
    let bad = format!("Basic tok_183");

    assert_eq!(parse_bearer_auth(&good), Some("tok_183"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_183"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_183() {
    let expected = if 183 < 9 { 2u64.pow(183u32) } else { 256 };
    assert_eq!(reconnect_backoff(183), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_184() {
    let transient = [
        "timeout #184",
        "connection reset by peer #184",
        "broken pipe #184",
        "temporarily unavailable #184",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #184",
        "permission denied #184",
        "bad request #184",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_184() {
    let good = format!("Bearer tok_184");
    let lower = format!("bearer tok_184");
    let bad = format!("Basic tok_184");

    assert_eq!(parse_bearer_auth(&good), Some("tok_184"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_184"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_184() {
    let expected = if 184 < 9 { 2u64.pow(184u32) } else { 256 };
    assert_eq!(reconnect_backoff(184), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_185() {
    let transient = [
        "timeout #185",
        "connection reset by peer #185",
        "broken pipe #185",
        "temporarily unavailable #185",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #185",
        "permission denied #185",
        "bad request #185",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_185() {
    let good = format!("Bearer tok_185");
    let lower = format!("bearer tok_185");
    let bad = format!("Basic tok_185");

    assert_eq!(parse_bearer_auth(&good), Some("tok_185"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_185"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_185() {
    let expected = if 185 < 9 { 2u64.pow(185u32) } else { 256 };
    assert_eq!(reconnect_backoff(185), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_186() {
    let transient = [
        "timeout #186",
        "connection reset by peer #186",
        "broken pipe #186",
        "temporarily unavailable #186",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #186",
        "permission denied #186",
        "bad request #186",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_186() {
    let good = format!("Bearer tok_186");
    let lower = format!("bearer tok_186");
    let bad = format!("Basic tok_186");

    assert_eq!(parse_bearer_auth(&good), Some("tok_186"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_186"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_186() {
    let expected = if 186 < 9 { 2u64.pow(186u32) } else { 256 };
    assert_eq!(reconnect_backoff(186), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_187() {
    let transient = [
        "timeout #187",
        "connection reset by peer #187",
        "broken pipe #187",
        "temporarily unavailable #187",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #187",
        "permission denied #187",
        "bad request #187",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_187() {
    let good = format!("Bearer tok_187");
    let lower = format!("bearer tok_187");
    let bad = format!("Basic tok_187");

    assert_eq!(parse_bearer_auth(&good), Some("tok_187"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_187"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_187() {
    let expected = if 187 < 9 { 2u64.pow(187u32) } else { 256 };
    assert_eq!(reconnect_backoff(187), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_188() {
    let transient = [
        "timeout #188",
        "connection reset by peer #188",
        "broken pipe #188",
        "temporarily unavailable #188",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #188",
        "permission denied #188",
        "bad request #188",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_188() {
    let good = format!("Bearer tok_188");
    let lower = format!("bearer tok_188");
    let bad = format!("Basic tok_188");

    assert_eq!(parse_bearer_auth(&good), Some("tok_188"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_188"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_188() {
    let expected = if 188 < 9 { 2u64.pow(188u32) } else { 256 };
    assert_eq!(reconnect_backoff(188), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_189() {
    let transient = [
        "timeout #189",
        "connection reset by peer #189",
        "broken pipe #189",
        "temporarily unavailable #189",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #189",
        "permission denied #189",
        "bad request #189",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_189() {
    let good = format!("Bearer tok_189");
    let lower = format!("bearer tok_189");
    let bad = format!("Basic tok_189");

    assert_eq!(parse_bearer_auth(&good), Some("tok_189"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_189"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_189() {
    let expected = if 189 < 9 { 2u64.pow(189u32) } else { 256 };
    assert_eq!(reconnect_backoff(189), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_190() {
    let transient = [
        "timeout #190",
        "connection reset by peer #190",
        "broken pipe #190",
        "temporarily unavailable #190",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #190",
        "permission denied #190",
        "bad request #190",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_190() {
    let good = format!("Bearer tok_190");
    let lower = format!("bearer tok_190");
    let bad = format!("Basic tok_190");

    assert_eq!(parse_bearer_auth(&good), Some("tok_190"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_190"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_190() {
    let expected = if 190 < 9 { 2u64.pow(190u32) } else { 256 };
    assert_eq!(reconnect_backoff(190), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_191() {
    let transient = [
        "timeout #191",
        "connection reset by peer #191",
        "broken pipe #191",
        "temporarily unavailable #191",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #191",
        "permission denied #191",
        "bad request #191",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_191() {
    let good = format!("Bearer tok_191");
    let lower = format!("bearer tok_191");
    let bad = format!("Basic tok_191");

    assert_eq!(parse_bearer_auth(&good), Some("tok_191"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_191"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_191() {
    let expected = if 191 < 9 { 2u64.pow(191u32) } else { 256 };
    assert_eq!(reconnect_backoff(191), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_192() {
    let transient = [
        "timeout #192",
        "connection reset by peer #192",
        "broken pipe #192",
        "temporarily unavailable #192",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #192",
        "permission denied #192",
        "bad request #192",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_192() {
    let good = format!("Bearer tok_192");
    let lower = format!("bearer tok_192");
    let bad = format!("Basic tok_192");

    assert_eq!(parse_bearer_auth(&good), Some("tok_192"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_192"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_192() {
    let expected = if 192 < 9 { 2u64.pow(192u32) } else { 256 };
    assert_eq!(reconnect_backoff(192), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_193() {
    let transient = [
        "timeout #193",
        "connection reset by peer #193",
        "broken pipe #193",
        "temporarily unavailable #193",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #193",
        "permission denied #193",
        "bad request #193",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_193() {
    let good = format!("Bearer tok_193");
    let lower = format!("bearer tok_193");
    let bad = format!("Basic tok_193");

    assert_eq!(parse_bearer_auth(&good), Some("tok_193"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_193"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_193() {
    let expected = if 193 < 9 { 2u64.pow(193u32) } else { 256 };
    assert_eq!(reconnect_backoff(193), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_194() {
    let transient = [
        "timeout #194",
        "connection reset by peer #194",
        "broken pipe #194",
        "temporarily unavailable #194",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #194",
        "permission denied #194",
        "bad request #194",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_194() {
    let good = format!("Bearer tok_194");
    let lower = format!("bearer tok_194");
    let bad = format!("Basic tok_194");

    assert_eq!(parse_bearer_auth(&good), Some("tok_194"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_194"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_194() {
    let expected = if 194 < 9 { 2u64.pow(194u32) } else { 256 };
    assert_eq!(reconnect_backoff(194), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_195() {
    let transient = [
        "timeout #195",
        "connection reset by peer #195",
        "broken pipe #195",
        "temporarily unavailable #195",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #195",
        "permission denied #195",
        "bad request #195",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_195() {
    let good = format!("Bearer tok_195");
    let lower = format!("bearer tok_195");
    let bad = format!("Basic tok_195");

    assert_eq!(parse_bearer_auth(&good), Some("tok_195"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_195"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_195() {
    let expected = if 195 < 9 { 2u64.pow(195u32) } else { 256 };
    assert_eq!(reconnect_backoff(195), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_196() {
    let transient = [
        "timeout #196",
        "connection reset by peer #196",
        "broken pipe #196",
        "temporarily unavailable #196",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #196",
        "permission denied #196",
        "bad request #196",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_196() {
    let good = format!("Bearer tok_196");
    let lower = format!("bearer tok_196");
    let bad = format!("Basic tok_196");

    assert_eq!(parse_bearer_auth(&good), Some("tok_196"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_196"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_196() {
    let expected = if 196 < 9 { 2u64.pow(196u32) } else { 256 };
    assert_eq!(reconnect_backoff(196), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_197() {
    let transient = [
        "timeout #197",
        "connection reset by peer #197",
        "broken pipe #197",
        "temporarily unavailable #197",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #197",
        "permission denied #197",
        "bad request #197",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_197() {
    let good = format!("Bearer tok_197");
    let lower = format!("bearer tok_197");
    let bad = format!("Basic tok_197");

    assert_eq!(parse_bearer_auth(&good), Some("tok_197"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_197"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_197() {
    let expected = if 197 < 9 { 2u64.pow(197u32) } else { 256 };
    assert_eq!(reconnect_backoff(197), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_198() {
    let transient = [
        "timeout #198",
        "connection reset by peer #198",
        "broken pipe #198",
        "temporarily unavailable #198",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #198",
        "permission denied #198",
        "bad request #198",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_198() {
    let good = format!("Bearer tok_198");
    let lower = format!("bearer tok_198");
    let bad = format!("Basic tok_198");

    assert_eq!(parse_bearer_auth(&good), Some("tok_198"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_198"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_198() {
    let expected = if 198 < 9 { 2u64.pow(198u32) } else { 256 };
    assert_eq!(reconnect_backoff(198), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_199() {
    let transient = [
        "timeout #199",
        "connection reset by peer #199",
        "broken pipe #199",
        "temporarily unavailable #199",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #199",
        "permission denied #199",
        "bad request #199",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_199() {
    let good = format!("Bearer tok_199");
    let lower = format!("bearer tok_199");
    let bad = format!("Basic tok_199");

    assert_eq!(parse_bearer_auth(&good), Some("tok_199"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_199"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_199() {
    let expected = if 199 < 9 { 2u64.pow(199u32) } else { 256 };
    assert_eq!(reconnect_backoff(199), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_200() {
    let transient = [
        "timeout #200",
        "connection reset by peer #200",
        "broken pipe #200",
        "temporarily unavailable #200",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #200",
        "permission denied #200",
        "bad request #200",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_200() {
    let good = format!("Bearer tok_200");
    let lower = format!("bearer tok_200");
    let bad = format!("Basic tok_200");

    assert_eq!(parse_bearer_auth(&good), Some("tok_200"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_200"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_200() {
    let expected = if 200 < 9 { 2u64.pow(200u32) } else { 256 };
    assert_eq!(reconnect_backoff(200), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_201() {
    let transient = [
        "timeout #201",
        "connection reset by peer #201",
        "broken pipe #201",
        "temporarily unavailable #201",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #201",
        "permission denied #201",
        "bad request #201",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_201() {
    let good = format!("Bearer tok_201");
    let lower = format!("bearer tok_201");
    let bad = format!("Basic tok_201");

    assert_eq!(parse_bearer_auth(&good), Some("tok_201"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_201"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_201() {
    let expected = if 201 < 9 { 2u64.pow(201u32) } else { 256 };
    assert_eq!(reconnect_backoff(201), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_202() {
    let transient = [
        "timeout #202",
        "connection reset by peer #202",
        "broken pipe #202",
        "temporarily unavailable #202",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #202",
        "permission denied #202",
        "bad request #202",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_202() {
    let good = format!("Bearer tok_202");
    let lower = format!("bearer tok_202");
    let bad = format!("Basic tok_202");

    assert_eq!(parse_bearer_auth(&good), Some("tok_202"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_202"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_202() {
    let expected = if 202 < 9 { 2u64.pow(202u32) } else { 256 };
    assert_eq!(reconnect_backoff(202), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_203() {
    let transient = [
        "timeout #203",
        "connection reset by peer #203",
        "broken pipe #203",
        "temporarily unavailable #203",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #203",
        "permission denied #203",
        "bad request #203",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_203() {
    let good = format!("Bearer tok_203");
    let lower = format!("bearer tok_203");
    let bad = format!("Basic tok_203");

    assert_eq!(parse_bearer_auth(&good), Some("tok_203"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_203"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_203() {
    let expected = if 203 < 9 { 2u64.pow(203u32) } else { 256 };
    assert_eq!(reconnect_backoff(203), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_204() {
    let transient = [
        "timeout #204",
        "connection reset by peer #204",
        "broken pipe #204",
        "temporarily unavailable #204",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #204",
        "permission denied #204",
        "bad request #204",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_204() {
    let good = format!("Bearer tok_204");
    let lower = format!("bearer tok_204");
    let bad = format!("Basic tok_204");

    assert_eq!(parse_bearer_auth(&good), Some("tok_204"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_204"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_204() {
    let expected = if 204 < 9 { 2u64.pow(204u32) } else { 256 };
    assert_eq!(reconnect_backoff(204), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_205() {
    let transient = [
        "timeout #205",
        "connection reset by peer #205",
        "broken pipe #205",
        "temporarily unavailable #205",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #205",
        "permission denied #205",
        "bad request #205",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_205() {
    let good = format!("Bearer tok_205");
    let lower = format!("bearer tok_205");
    let bad = format!("Basic tok_205");

    assert_eq!(parse_bearer_auth(&good), Some("tok_205"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_205"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_205() {
    let expected = if 205 < 9 { 2u64.pow(205u32) } else { 256 };
    assert_eq!(reconnect_backoff(205), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_206() {
    let transient = [
        "timeout #206",
        "connection reset by peer #206",
        "broken pipe #206",
        "temporarily unavailable #206",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #206",
        "permission denied #206",
        "bad request #206",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_206() {
    let good = format!("Bearer tok_206");
    let lower = format!("bearer tok_206");
    let bad = format!("Basic tok_206");

    assert_eq!(parse_bearer_auth(&good), Some("tok_206"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_206"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_206() {
    let expected = if 206 < 9 { 2u64.pow(206u32) } else { 256 };
    assert_eq!(reconnect_backoff(206), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_207() {
    let transient = [
        "timeout #207",
        "connection reset by peer #207",
        "broken pipe #207",
        "temporarily unavailable #207",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #207",
        "permission denied #207",
        "bad request #207",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_207() {
    let good = format!("Bearer tok_207");
    let lower = format!("bearer tok_207");
    let bad = format!("Basic tok_207");

    assert_eq!(parse_bearer_auth(&good), Some("tok_207"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_207"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_207() {
    let expected = if 207 < 9 { 2u64.pow(207u32) } else { 256 };
    assert_eq!(reconnect_backoff(207), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_208() {
    let transient = [
        "timeout #208",
        "connection reset by peer #208",
        "broken pipe #208",
        "temporarily unavailable #208",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #208",
        "permission denied #208",
        "bad request #208",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_208() {
    let good = format!("Bearer tok_208");
    let lower = format!("bearer tok_208");
    let bad = format!("Basic tok_208");

    assert_eq!(parse_bearer_auth(&good), Some("tok_208"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_208"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_208() {
    let expected = if 208 < 9 { 2u64.pow(208u32) } else { 256 };
    assert_eq!(reconnect_backoff(208), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_209() {
    let transient = [
        "timeout #209",
        "connection reset by peer #209",
        "broken pipe #209",
        "temporarily unavailable #209",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #209",
        "permission denied #209",
        "bad request #209",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_209() {
    let good = format!("Bearer tok_209");
    let lower = format!("bearer tok_209");
    let bad = format!("Basic tok_209");

    assert_eq!(parse_bearer_auth(&good), Some("tok_209"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_209"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_209() {
    let expected = if 209 < 9 { 2u64.pow(209u32) } else { 256 };
    assert_eq!(reconnect_backoff(209), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_210() {
    let transient = [
        "timeout #210",
        "connection reset by peer #210",
        "broken pipe #210",
        "temporarily unavailable #210",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #210",
        "permission denied #210",
        "bad request #210",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_210() {
    let good = format!("Bearer tok_210");
    let lower = format!("bearer tok_210");
    let bad = format!("Basic tok_210");

    assert_eq!(parse_bearer_auth(&good), Some("tok_210"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_210"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_210() {
    let expected = if 210 < 9 { 2u64.pow(210u32) } else { 256 };
    assert_eq!(reconnect_backoff(210), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_211() {
    let transient = [
        "timeout #211",
        "connection reset by peer #211",
        "broken pipe #211",
        "temporarily unavailable #211",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #211",
        "permission denied #211",
        "bad request #211",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_211() {
    let good = format!("Bearer tok_211");
    let lower = format!("bearer tok_211");
    let bad = format!("Basic tok_211");

    assert_eq!(parse_bearer_auth(&good), Some("tok_211"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_211"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_211() {
    let expected = if 211 < 9 { 2u64.pow(211u32) } else { 256 };
    assert_eq!(reconnect_backoff(211), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_212() {
    let transient = [
        "timeout #212",
        "connection reset by peer #212",
        "broken pipe #212",
        "temporarily unavailable #212",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #212",
        "permission denied #212",
        "bad request #212",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_212() {
    let good = format!("Bearer tok_212");
    let lower = format!("bearer tok_212");
    let bad = format!("Basic tok_212");

    assert_eq!(parse_bearer_auth(&good), Some("tok_212"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_212"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_212() {
    let expected = if 212 < 9 { 2u64.pow(212u32) } else { 256 };
    assert_eq!(reconnect_backoff(212), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_213() {
    let transient = [
        "timeout #213",
        "connection reset by peer #213",
        "broken pipe #213",
        "temporarily unavailable #213",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #213",
        "permission denied #213",
        "bad request #213",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_213() {
    let good = format!("Bearer tok_213");
    let lower = format!("bearer tok_213");
    let bad = format!("Basic tok_213");

    assert_eq!(parse_bearer_auth(&good), Some("tok_213"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_213"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_213() {
    let expected = if 213 < 9 { 2u64.pow(213u32) } else { 256 };
    assert_eq!(reconnect_backoff(213), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_214() {
    let transient = [
        "timeout #214",
        "connection reset by peer #214",
        "broken pipe #214",
        "temporarily unavailable #214",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #214",
        "permission denied #214",
        "bad request #214",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_214() {
    let good = format!("Bearer tok_214");
    let lower = format!("bearer tok_214");
    let bad = format!("Basic tok_214");

    assert_eq!(parse_bearer_auth(&good), Some("tok_214"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_214"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_214() {
    let expected = if 214 < 9 { 2u64.pow(214u32) } else { 256 };
    assert_eq!(reconnect_backoff(214), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_215() {
    let transient = [
        "timeout #215",
        "connection reset by peer #215",
        "broken pipe #215",
        "temporarily unavailable #215",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #215",
        "permission denied #215",
        "bad request #215",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_215() {
    let good = format!("Bearer tok_215");
    let lower = format!("bearer tok_215");
    let bad = format!("Basic tok_215");

    assert_eq!(parse_bearer_auth(&good), Some("tok_215"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_215"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_215() {
    let expected = if 215 < 9 { 2u64.pow(215u32) } else { 256 };
    assert_eq!(reconnect_backoff(215), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_216() {
    let transient = [
        "timeout #216",
        "connection reset by peer #216",
        "broken pipe #216",
        "temporarily unavailable #216",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #216",
        "permission denied #216",
        "bad request #216",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_216() {
    let good = format!("Bearer tok_216");
    let lower = format!("bearer tok_216");
    let bad = format!("Basic tok_216");

    assert_eq!(parse_bearer_auth(&good), Some("tok_216"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_216"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_216() {
    let expected = if 216 < 9 { 2u64.pow(216u32) } else { 256 };
    assert_eq!(reconnect_backoff(216), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_217() {
    let transient = [
        "timeout #217",
        "connection reset by peer #217",
        "broken pipe #217",
        "temporarily unavailable #217",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #217",
        "permission denied #217",
        "bad request #217",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_217() {
    let good = format!("Bearer tok_217");
    let lower = format!("bearer tok_217");
    let bad = format!("Basic tok_217");

    assert_eq!(parse_bearer_auth(&good), Some("tok_217"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_217"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_217() {
    let expected = if 217 < 9 { 2u64.pow(217u32) } else { 256 };
    assert_eq!(reconnect_backoff(217), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_218() {
    let transient = [
        "timeout #218",
        "connection reset by peer #218",
        "broken pipe #218",
        "temporarily unavailable #218",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #218",
        "permission denied #218",
        "bad request #218",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_218() {
    let good = format!("Bearer tok_218");
    let lower = format!("bearer tok_218");
    let bad = format!("Basic tok_218");

    assert_eq!(parse_bearer_auth(&good), Some("tok_218"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_218"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_218() {
    let expected = if 218 < 9 { 2u64.pow(218u32) } else { 256 };
    assert_eq!(reconnect_backoff(218), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_219() {
    let transient = [
        "timeout #219",
        "connection reset by peer #219",
        "broken pipe #219",
        "temporarily unavailable #219",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #219",
        "permission denied #219",
        "bad request #219",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_219() {
    let good = format!("Bearer tok_219");
    let lower = format!("bearer tok_219");
    let bad = format!("Basic tok_219");

    assert_eq!(parse_bearer_auth(&good), Some("tok_219"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_219"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_219() {
    let expected = if 219 < 9 { 2u64.pow(219u32) } else { 256 };
    assert_eq!(reconnect_backoff(219), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_220() {
    let transient = [
        "timeout #220",
        "connection reset by peer #220",
        "broken pipe #220",
        "temporarily unavailable #220",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #220",
        "permission denied #220",
        "bad request #220",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_220() {
    let good = format!("Bearer tok_220");
    let lower = format!("bearer tok_220");
    let bad = format!("Basic tok_220");

    assert_eq!(parse_bearer_auth(&good), Some("tok_220"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_220"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_220() {
    let expected = if 220 < 9 { 2u64.pow(220u32) } else { 256 };
    assert_eq!(reconnect_backoff(220), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_221() {
    let transient = [
        "timeout #221",
        "connection reset by peer #221",
        "broken pipe #221",
        "temporarily unavailable #221",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #221",
        "permission denied #221",
        "bad request #221",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_221() {
    let good = format!("Bearer tok_221");
    let lower = format!("bearer tok_221");
    let bad = format!("Basic tok_221");

    assert_eq!(parse_bearer_auth(&good), Some("tok_221"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_221"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_221() {
    let expected = if 221 < 9 { 2u64.pow(221u32) } else { 256 };
    assert_eq!(reconnect_backoff(221), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_222() {
    let transient = [
        "timeout #222",
        "connection reset by peer #222",
        "broken pipe #222",
        "temporarily unavailable #222",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #222",
        "permission denied #222",
        "bad request #222",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_222() {
    let good = format!("Bearer tok_222");
    let lower = format!("bearer tok_222");
    let bad = format!("Basic tok_222");

    assert_eq!(parse_bearer_auth(&good), Some("tok_222"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_222"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_222() {
    let expected = if 222 < 9 { 2u64.pow(222u32) } else { 256 };
    assert_eq!(reconnect_backoff(222), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_223() {
    let transient = [
        "timeout #223",
        "connection reset by peer #223",
        "broken pipe #223",
        "temporarily unavailable #223",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #223",
        "permission denied #223",
        "bad request #223",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_223() {
    let good = format!("Bearer tok_223");
    let lower = format!("bearer tok_223");
    let bad = format!("Basic tok_223");

    assert_eq!(parse_bearer_auth(&good), Some("tok_223"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_223"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_223() {
    let expected = if 223 < 9 { 2u64.pow(223u32) } else { 256 };
    assert_eq!(reconnect_backoff(223), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_224() {
    let transient = [
        "timeout #224",
        "connection reset by peer #224",
        "broken pipe #224",
        "temporarily unavailable #224",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #224",
        "permission denied #224",
        "bad request #224",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_224() {
    let good = format!("Bearer tok_224");
    let lower = format!("bearer tok_224");
    let bad = format!("Basic tok_224");

    assert_eq!(parse_bearer_auth(&good), Some("tok_224"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_224"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_224() {
    let expected = if 224 < 9 { 2u64.pow(224u32) } else { 256 };
    assert_eq!(reconnect_backoff(224), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_225() {
    let transient = [
        "timeout #225",
        "connection reset by peer #225",
        "broken pipe #225",
        "temporarily unavailable #225",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #225",
        "permission denied #225",
        "bad request #225",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_225() {
    let good = format!("Bearer tok_225");
    let lower = format!("bearer tok_225");
    let bad = format!("Basic tok_225");

    assert_eq!(parse_bearer_auth(&good), Some("tok_225"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_225"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_225() {
    let expected = if 225 < 9 { 2u64.pow(225u32) } else { 256 };
    assert_eq!(reconnect_backoff(225), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_226() {
    let transient = [
        "timeout #226",
        "connection reset by peer #226",
        "broken pipe #226",
        "temporarily unavailable #226",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #226",
        "permission denied #226",
        "bad request #226",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_226() {
    let good = format!("Bearer tok_226");
    let lower = format!("bearer tok_226");
    let bad = format!("Basic tok_226");

    assert_eq!(parse_bearer_auth(&good), Some("tok_226"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_226"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_226() {
    let expected = if 226 < 9 { 2u64.pow(226u32) } else { 256 };
    assert_eq!(reconnect_backoff(226), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_227() {
    let transient = [
        "timeout #227",
        "connection reset by peer #227",
        "broken pipe #227",
        "temporarily unavailable #227",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #227",
        "permission denied #227",
        "bad request #227",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_227() {
    let good = format!("Bearer tok_227");
    let lower = format!("bearer tok_227");
    let bad = format!("Basic tok_227");

    assert_eq!(parse_bearer_auth(&good), Some("tok_227"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_227"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_227() {
    let expected = if 227 < 9 { 2u64.pow(227u32) } else { 256 };
    assert_eq!(reconnect_backoff(227), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_228() {
    let transient = [
        "timeout #228",
        "connection reset by peer #228",
        "broken pipe #228",
        "temporarily unavailable #228",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #228",
        "permission denied #228",
        "bad request #228",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_228() {
    let good = format!("Bearer tok_228");
    let lower = format!("bearer tok_228");
    let bad = format!("Basic tok_228");

    assert_eq!(parse_bearer_auth(&good), Some("tok_228"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_228"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_228() {
    let expected = if 228 < 9 { 2u64.pow(228u32) } else { 256 };
    assert_eq!(reconnect_backoff(228), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_229() {
    let transient = [
        "timeout #229",
        "connection reset by peer #229",
        "broken pipe #229",
        "temporarily unavailable #229",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #229",
        "permission denied #229",
        "bad request #229",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_229() {
    let good = format!("Bearer tok_229");
    let lower = format!("bearer tok_229");
    let bad = format!("Basic tok_229");

    assert_eq!(parse_bearer_auth(&good), Some("tok_229"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_229"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_229() {
    let expected = if 229 < 9 { 2u64.pow(229u32) } else { 256 };
    assert_eq!(reconnect_backoff(229), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_230() {
    let transient = [
        "timeout #230",
        "connection reset by peer #230",
        "broken pipe #230",
        "temporarily unavailable #230",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #230",
        "permission denied #230",
        "bad request #230",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_230() {
    let good = format!("Bearer tok_230");
    let lower = format!("bearer tok_230");
    let bad = format!("Basic tok_230");

    assert_eq!(parse_bearer_auth(&good), Some("tok_230"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_230"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_230() {
    let expected = if 230 < 9 { 2u64.pow(230u32) } else { 256 };
    assert_eq!(reconnect_backoff(230), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_231() {
    let transient = [
        "timeout #231",
        "connection reset by peer #231",
        "broken pipe #231",
        "temporarily unavailable #231",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #231",
        "permission denied #231",
        "bad request #231",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_231() {
    let good = format!("Bearer tok_231");
    let lower = format!("bearer tok_231");
    let bad = format!("Basic tok_231");

    assert_eq!(parse_bearer_auth(&good), Some("tok_231"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_231"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_231() {
    let expected = if 231 < 9 { 2u64.pow(231u32) } else { 256 };
    assert_eq!(reconnect_backoff(231), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_232() {
    let transient = [
        "timeout #232",
        "connection reset by peer #232",
        "broken pipe #232",
        "temporarily unavailable #232",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #232",
        "permission denied #232",
        "bad request #232",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_232() {
    let good = format!("Bearer tok_232");
    let lower = format!("bearer tok_232");
    let bad = format!("Basic tok_232");

    assert_eq!(parse_bearer_auth(&good), Some("tok_232"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_232"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_232() {
    let expected = if 232 < 9 { 2u64.pow(232u32) } else { 256 };
    assert_eq!(reconnect_backoff(232), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_233() {
    let transient = [
        "timeout #233",
        "connection reset by peer #233",
        "broken pipe #233",
        "temporarily unavailable #233",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #233",
        "permission denied #233",
        "bad request #233",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_233() {
    let good = format!("Bearer tok_233");
    let lower = format!("bearer tok_233");
    let bad = format!("Basic tok_233");

    assert_eq!(parse_bearer_auth(&good), Some("tok_233"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_233"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_233() {
    let expected = if 233 < 9 { 2u64.pow(233u32) } else { 256 };
    assert_eq!(reconnect_backoff(233), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_234() {
    let transient = [
        "timeout #234",
        "connection reset by peer #234",
        "broken pipe #234",
        "temporarily unavailable #234",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #234",
        "permission denied #234",
        "bad request #234",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_234() {
    let good = format!("Bearer tok_234");
    let lower = format!("bearer tok_234");
    let bad = format!("Basic tok_234");

    assert_eq!(parse_bearer_auth(&good), Some("tok_234"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_234"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_234() {
    let expected = if 234 < 9 { 2u64.pow(234u32) } else { 256 };
    assert_eq!(reconnect_backoff(234), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_235() {
    let transient = [
        "timeout #235",
        "connection reset by peer #235",
        "broken pipe #235",
        "temporarily unavailable #235",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #235",
        "permission denied #235",
        "bad request #235",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_235() {
    let good = format!("Bearer tok_235");
    let lower = format!("bearer tok_235");
    let bad = format!("Basic tok_235");

    assert_eq!(parse_bearer_auth(&good), Some("tok_235"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_235"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_235() {
    let expected = if 235 < 9 { 2u64.pow(235u32) } else { 256 };
    assert_eq!(reconnect_backoff(235), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_236() {
    let transient = [
        "timeout #236",
        "connection reset by peer #236",
        "broken pipe #236",
        "temporarily unavailable #236",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #236",
        "permission denied #236",
        "bad request #236",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_236() {
    let good = format!("Bearer tok_236");
    let lower = format!("bearer tok_236");
    let bad = format!("Basic tok_236");

    assert_eq!(parse_bearer_auth(&good), Some("tok_236"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_236"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_236() {
    let expected = if 236 < 9 { 2u64.pow(236u32) } else { 256 };
    assert_eq!(reconnect_backoff(236), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_237() {
    let transient = [
        "timeout #237",
        "connection reset by peer #237",
        "broken pipe #237",
        "temporarily unavailable #237",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #237",
        "permission denied #237",
        "bad request #237",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_237() {
    let good = format!("Bearer tok_237");
    let lower = format!("bearer tok_237");
    let bad = format!("Basic tok_237");

    assert_eq!(parse_bearer_auth(&good), Some("tok_237"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_237"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_237() {
    let expected = if 237 < 9 { 2u64.pow(237u32) } else { 256 };
    assert_eq!(reconnect_backoff(237), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_238() {
    let transient = [
        "timeout #238",
        "connection reset by peer #238",
        "broken pipe #238",
        "temporarily unavailable #238",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #238",
        "permission denied #238",
        "bad request #238",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_238() {
    let good = format!("Bearer tok_238");
    let lower = format!("bearer tok_238");
    let bad = format!("Basic tok_238");

    assert_eq!(parse_bearer_auth(&good), Some("tok_238"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_238"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_238() {
    let expected = if 238 < 9 { 2u64.pow(238u32) } else { 256 };
    assert_eq!(reconnect_backoff(238), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_239() {
    let transient = [
        "timeout #239",
        "connection reset by peer #239",
        "broken pipe #239",
        "temporarily unavailable #239",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #239",
        "permission denied #239",
        "bad request #239",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_239() {
    let good = format!("Bearer tok_239");
    let lower = format!("bearer tok_239");
    let bad = format!("Basic tok_239");

    assert_eq!(parse_bearer_auth(&good), Some("tok_239"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_239"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_239() {
    let expected = if 239 < 9 { 2u64.pow(239u32) } else { 256 };
    assert_eq!(reconnect_backoff(239), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_240() {
    let transient = [
        "timeout #240",
        "connection reset by peer #240",
        "broken pipe #240",
        "temporarily unavailable #240",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #240",
        "permission denied #240",
        "bad request #240",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_240() {
    let good = format!("Bearer tok_240");
    let lower = format!("bearer tok_240");
    let bad = format!("Basic tok_240");

    assert_eq!(parse_bearer_auth(&good), Some("tok_240"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_240"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_240() {
    let expected = if 240 < 9 { 2u64.pow(240u32) } else { 256 };
    assert_eq!(reconnect_backoff(240), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_241() {
    let transient = [
        "timeout #241",
        "connection reset by peer #241",
        "broken pipe #241",
        "temporarily unavailable #241",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #241",
        "permission denied #241",
        "bad request #241",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_241() {
    let good = format!("Bearer tok_241");
    let lower = format!("bearer tok_241");
    let bad = format!("Basic tok_241");

    assert_eq!(parse_bearer_auth(&good), Some("tok_241"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_241"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_241() {
    let expected = if 241 < 9 { 2u64.pow(241u32) } else { 256 };
    assert_eq!(reconnect_backoff(241), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_242() {
    let transient = [
        "timeout #242",
        "connection reset by peer #242",
        "broken pipe #242",
        "temporarily unavailable #242",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #242",
        "permission denied #242",
        "bad request #242",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_242() {
    let good = format!("Bearer tok_242");
    let lower = format!("bearer tok_242");
    let bad = format!("Basic tok_242");

    assert_eq!(parse_bearer_auth(&good), Some("tok_242"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_242"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_242() {
    let expected = if 242 < 9 { 2u64.pow(242u32) } else { 256 };
    assert_eq!(reconnect_backoff(242), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_243() {
    let transient = [
        "timeout #243",
        "connection reset by peer #243",
        "broken pipe #243",
        "temporarily unavailable #243",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #243",
        "permission denied #243",
        "bad request #243",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_243() {
    let good = format!("Bearer tok_243");
    let lower = format!("bearer tok_243");
    let bad = format!("Basic tok_243");

    assert_eq!(parse_bearer_auth(&good), Some("tok_243"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_243"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_243() {
    let expected = if 243 < 9 { 2u64.pow(243u32) } else { 256 };
    assert_eq!(reconnect_backoff(243), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_244() {
    let transient = [
        "timeout #244",
        "connection reset by peer #244",
        "broken pipe #244",
        "temporarily unavailable #244",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #244",
        "permission denied #244",
        "bad request #244",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_244() {
    let good = format!("Bearer tok_244");
    let lower = format!("bearer tok_244");
    let bad = format!("Basic tok_244");

    assert_eq!(parse_bearer_auth(&good), Some("tok_244"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_244"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_244() {
    let expected = if 244 < 9 { 2u64.pow(244u32) } else { 256 };
    assert_eq!(reconnect_backoff(244), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_245() {
    let transient = [
        "timeout #245",
        "connection reset by peer #245",
        "broken pipe #245",
        "temporarily unavailable #245",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #245",
        "permission denied #245",
        "bad request #245",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_245() {
    let good = format!("Bearer tok_245");
    let lower = format!("bearer tok_245");
    let bad = format!("Basic tok_245");

    assert_eq!(parse_bearer_auth(&good), Some("tok_245"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_245"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_245() {
    let expected = if 245 < 9 { 2u64.pow(245u32) } else { 256 };
    assert_eq!(reconnect_backoff(245), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_246() {
    let transient = [
        "timeout #246",
        "connection reset by peer #246",
        "broken pipe #246",
        "temporarily unavailable #246",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #246",
        "permission denied #246",
        "bad request #246",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_246() {
    let good = format!("Bearer tok_246");
    let lower = format!("bearer tok_246");
    let bad = format!("Basic tok_246");

    assert_eq!(parse_bearer_auth(&good), Some("tok_246"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_246"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_246() {
    let expected = if 246 < 9 { 2u64.pow(246u32) } else { 256 };
    assert_eq!(reconnect_backoff(246), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_247() {
    let transient = [
        "timeout #247",
        "connection reset by peer #247",
        "broken pipe #247",
        "temporarily unavailable #247",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #247",
        "permission denied #247",
        "bad request #247",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_247() {
    let good = format!("Bearer tok_247");
    let lower = format!("bearer tok_247");
    let bad = format!("Basic tok_247");

    assert_eq!(parse_bearer_auth(&good), Some("tok_247"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_247"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_247() {
    let expected = if 247 < 9 { 2u64.pow(247u32) } else { 256 };
    assert_eq!(reconnect_backoff(247), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_248() {
    let transient = [
        "timeout #248",
        "connection reset by peer #248",
        "broken pipe #248",
        "temporarily unavailable #248",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #248",
        "permission denied #248",
        "bad request #248",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_248() {
    let good = format!("Bearer tok_248");
    let lower = format!("bearer tok_248");
    let bad = format!("Basic tok_248");

    assert_eq!(parse_bearer_auth(&good), Some("tok_248"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_248"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_248() {
    let expected = if 248 < 9 { 2u64.pow(248u32) } else { 256 };
    assert_eq!(reconnect_backoff(248), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_249() {
    let transient = [
        "timeout #249",
        "connection reset by peer #249",
        "broken pipe #249",
        "temporarily unavailable #249",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #249",
        "permission denied #249",
        "bad request #249",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_249() {
    let good = format!("Bearer tok_249");
    let lower = format!("bearer tok_249");
    let bad = format!("Basic tok_249");

    assert_eq!(parse_bearer_auth(&good), Some("tok_249"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_249"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_249() {
    let expected = if 249 < 9 { 2u64.pow(249u32) } else { 256 };
    assert_eq!(reconnect_backoff(249), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_250() {
    let transient = [
        "timeout #250",
        "connection reset by peer #250",
        "broken pipe #250",
        "temporarily unavailable #250",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #250",
        "permission denied #250",
        "bad request #250",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_250() {
    let good = format!("Bearer tok_250");
    let lower = format!("bearer tok_250");
    let bad = format!("Basic tok_250");

    assert_eq!(parse_bearer_auth(&good), Some("tok_250"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_250"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_250() {
    let expected = if 250 < 9 { 2u64.pow(250u32) } else { 256 };
    assert_eq!(reconnect_backoff(250), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_251() {
    let transient = [
        "timeout #251",
        "connection reset by peer #251",
        "broken pipe #251",
        "temporarily unavailable #251",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #251",
        "permission denied #251",
        "bad request #251",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_251() {
    let good = format!("Bearer tok_251");
    let lower = format!("bearer tok_251");
    let bad = format!("Basic tok_251");

    assert_eq!(parse_bearer_auth(&good), Some("tok_251"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_251"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_251() {
    let expected = if 251 < 9 { 2u64.pow(251u32) } else { 256 };
    assert_eq!(reconnect_backoff(251), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_252() {
    let transient = [
        "timeout #252",
        "connection reset by peer #252",
        "broken pipe #252",
        "temporarily unavailable #252",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #252",
        "permission denied #252",
        "bad request #252",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_252() {
    let good = format!("Bearer tok_252");
    let lower = format!("bearer tok_252");
    let bad = format!("Basic tok_252");

    assert_eq!(parse_bearer_auth(&good), Some("tok_252"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_252"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_252() {
    let expected = if 252 < 9 { 2u64.pow(252u32) } else { 256 };
    assert_eq!(reconnect_backoff(252), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_253() {
    let transient = [
        "timeout #253",
        "connection reset by peer #253",
        "broken pipe #253",
        "temporarily unavailable #253",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #253",
        "permission denied #253",
        "bad request #253",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_253() {
    let good = format!("Bearer tok_253");
    let lower = format!("bearer tok_253");
    let bad = format!("Basic tok_253");

    assert_eq!(parse_bearer_auth(&good), Some("tok_253"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_253"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_253() {
    let expected = if 253 < 9 { 2u64.pow(253u32) } else { 256 };
    assert_eq!(reconnect_backoff(253), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_254() {
    let transient = [
        "timeout #254",
        "connection reset by peer #254",
        "broken pipe #254",
        "temporarily unavailable #254",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #254",
        "permission denied #254",
        "bad request #254",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_254() {
    let good = format!("Bearer tok_254");
    let lower = format!("bearer tok_254");
    let bad = format!("Basic tok_254");

    assert_eq!(parse_bearer_auth(&good), Some("tok_254"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_254"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_254() {
    let expected = if 254 < 9 { 2u64.pow(254u32) } else { 256 };
    assert_eq!(reconnect_backoff(254), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_255() {
    let transient = [
        "timeout #255",
        "connection reset by peer #255",
        "broken pipe #255",
        "temporarily unavailable #255",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #255",
        "permission denied #255",
        "bad request #255",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_255() {
    let good = format!("Bearer tok_255");
    let lower = format!("bearer tok_255");
    let bad = format!("Basic tok_255");

    assert_eq!(parse_bearer_auth(&good), Some("tok_255"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_255"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_255() {
    let expected = if 255 < 9 { 2u64.pow(255u32) } else { 256 };
    assert_eq!(reconnect_backoff(255), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_256() {
    let transient = [
        "timeout #256",
        "connection reset by peer #256",
        "broken pipe #256",
        "temporarily unavailable #256",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #256",
        "permission denied #256",
        "bad request #256",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_256() {
    let good = format!("Bearer tok_256");
    let lower = format!("bearer tok_256");
    let bad = format!("Basic tok_256");

    assert_eq!(parse_bearer_auth(&good), Some("tok_256"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_256"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_256() {
    let expected = if 256 < 9 { 2u64.pow(256u32) } else { 256 };
    assert_eq!(reconnect_backoff(256), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_257() {
    let transient = [
        "timeout #257",
        "connection reset by peer #257",
        "broken pipe #257",
        "temporarily unavailable #257",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #257",
        "permission denied #257",
        "bad request #257",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_257() {
    let good = format!("Bearer tok_257");
    let lower = format!("bearer tok_257");
    let bad = format!("Basic tok_257");

    assert_eq!(parse_bearer_auth(&good), Some("tok_257"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_257"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_257() {
    let expected = if 257 < 9 { 2u64.pow(257u32) } else { 256 };
    assert_eq!(reconnect_backoff(257), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_258() {
    let transient = [
        "timeout #258",
        "connection reset by peer #258",
        "broken pipe #258",
        "temporarily unavailable #258",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #258",
        "permission denied #258",
        "bad request #258",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_258() {
    let good = format!("Bearer tok_258");
    let lower = format!("bearer tok_258");
    let bad = format!("Basic tok_258");

    assert_eq!(parse_bearer_auth(&good), Some("tok_258"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_258"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_258() {
    let expected = if 258 < 9 { 2u64.pow(258u32) } else { 256 };
    assert_eq!(reconnect_backoff(258), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_259() {
    let transient = [
        "timeout #259",
        "connection reset by peer #259",
        "broken pipe #259",
        "temporarily unavailable #259",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #259",
        "permission denied #259",
        "bad request #259",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_259() {
    let good = format!("Bearer tok_259");
    let lower = format!("bearer tok_259");
    let bad = format!("Basic tok_259");

    assert_eq!(parse_bearer_auth(&good), Some("tok_259"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_259"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_259() {
    let expected = if 259 < 9 { 2u64.pow(259u32) } else { 256 };
    assert_eq!(reconnect_backoff(259), Duration::from_secs(expected));
}

#[test]
fn retry_policy_case_260() {
    let transient = [
        "timeout #260",
        "connection reset by peer #260",
        "broken pipe #260",
        "temporarily unavailable #260",
    ];
    for item in transient {
        assert!(should_retry_connection(item), "expected retry for {item}");
    }

    let terminal = [
        "invalid auth token #260",
        "permission denied #260",
        "bad request #260",
    ];
    for item in terminal {
        assert!(!should_retry_connection(item), "expected no retry for {item}");
    }
}

#[test]
fn bearer_parser_case_260() {
    let good = format!("Bearer tok_260");
    let lower = format!("bearer tok_260");
    let bad = format!("Basic tok_260");

    assert_eq!(parse_bearer_auth(&good), Some("tok_260"));
    assert_eq!(parse_bearer_auth(&lower), Some("tok_260"));
    assert_eq!(parse_bearer_auth(&bad), None);
}

#[test]
fn backoff_case_260() {
    let expected = if 260 < 9 { 2u64.pow(260u32) } else { 256 };
    assert_eq!(reconnect_backoff(260), Duration::from_secs(expected));
}

#[tokio::test]
async fn queue_matrix_case_1() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-1");
    let r1 = format!("1-r1");
    let r2 = format!("1-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_2() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-2");
    let r1 = format!("2-r1");
    let r2 = format!("2-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_3() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-3");
    let r1 = format!("3-r1");
    let r2 = format!("3-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_4() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-4");
    let r1 = format!("4-r1");
    let r2 = format!("4-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_5() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-5");
    let r1 = format!("5-r1");
    let r2 = format!("5-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_6() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-6");
    let r1 = format!("6-r1");
    let r2 = format!("6-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_7() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-7");
    let r1 = format!("7-r1");
    let r2 = format!("7-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_8() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-8");
    let r1 = format!("8-r1");
    let r2 = format!("8-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_9() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-9");
    let r1 = format!("9-r1");
    let r2 = format!("9-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_10() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-10");
    let r1 = format!("10-r1");
    let r2 = format!("10-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_11() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-11");
    let r1 = format!("11-r1");
    let r2 = format!("11-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_12() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-12");
    let r1 = format!("12-r1");
    let r2 = format!("12-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_13() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-13");
    let r1 = format!("13-r1");
    let r2 = format!("13-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_14() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-14");
    let r1 = format!("14-r1");
    let r2 = format!("14-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_15() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-15");
    let r1 = format!("15-r1");
    let r2 = format!("15-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_16() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-16");
    let r1 = format!("16-r1");
    let r2 = format!("16-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_17() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-17");
    let r1 = format!("17-r1");
    let r2 = format!("17-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_18() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-18");
    let r1 = format!("18-r1");
    let r2 = format!("18-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_19() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-19");
    let r1 = format!("19-r1");
    let r2 = format!("19-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_20() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-20");
    let r1 = format!("20-r1");
    let r2 = format!("20-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_21() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-21");
    let r1 = format!("21-r1");
    let r2 = format!("21-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_22() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-22");
    let r1 = format!("22-r1");
    let r2 = format!("22-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_23() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-23");
    let r1 = format!("23-r1");
    let r2 = format!("23-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_24() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-24");
    let r1 = format!("24-r1");
    let r2 = format!("24-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_25() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-25");
    let r1 = format!("25-r1");
    let r2 = format!("25-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_26() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-26");
    let r1 = format!("26-r1");
    let r2 = format!("26-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_27() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-27");
    let r1 = format!("27-r1");
    let r2 = format!("27-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_28() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-28");
    let r1 = format!("28-r1");
    let r2 = format!("28-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_29() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-29");
    let r1 = format!("29-r1");
    let r2 = format!("29-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_30() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-30");
    let r1 = format!("30-r1");
    let r2 = format!("30-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_31() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-31");
    let r1 = format!("31-r1");
    let r2 = format!("31-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_32() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-32");
    let r1 = format!("32-r1");
    let r2 = format!("32-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_33() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-33");
    let r1 = format!("33-r1");
    let r2 = format!("33-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_34() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-34");
    let r1 = format!("34-r1");
    let r2 = format!("34-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_35() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-35");
    let r1 = format!("35-r1");
    let r2 = format!("35-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_36() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-36");
    let r1 = format!("36-r1");
    let r2 = format!("36-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_37() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-37");
    let r1 = format!("37-r1");
    let r2 = format!("37-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_38() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-38");
    let r1 = format!("38-r1");
    let r2 = format!("38-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_39() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-39");
    let r1 = format!("39-r1");
    let r2 = format!("39-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_40() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-40");
    let r1 = format!("40-r1");
    let r2 = format!("40-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_41() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-41");
    let r1 = format!("41-r1");
    let r2 = format!("41-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_42() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-42");
    let r1 = format!("42-r1");
    let r2 = format!("42-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_43() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-43");
    let r1 = format!("43-r1");
    let r2 = format!("43-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_44() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-44");
    let r1 = format!("44-r1");
    let r2 = format!("44-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_45() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-45");
    let r1 = format!("45-r1");
    let r2 = format!("45-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_46() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-46");
    let r1 = format!("46-r1");
    let r2 = format!("46-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_47() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-47");
    let r1 = format!("47-r1");
    let r2 = format!("47-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_48() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-48");
    let r1 = format!("48-r1");
    let r2 = format!("48-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_49() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-49");
    let r1 = format!("49-r1");
    let r2 = format!("49-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_50() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-50");
    let r1 = format!("50-r1");
    let r2 = format!("50-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_51() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-51");
    let r1 = format!("51-r1");
    let r2 = format!("51-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_52() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-52");
    let r1 = format!("52-r1");
    let r2 = format!("52-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_53() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-53");
    let r1 = format!("53-r1");
    let r2 = format!("53-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_54() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-54");
    let r1 = format!("54-r1");
    let r2 = format!("54-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_55() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-55");
    let r1 = format!("55-r1");
    let r2 = format!("55-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_56() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-56");
    let r1 = format!("56-r1");
    let r2 = format!("56-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_57() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-57");
    let r1 = format!("57-r1");
    let r2 = format!("57-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_58() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-58");
    let r1 = format!("58-r1");
    let r2 = format!("58-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_59() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-59");
    let r1 = format!("59-r1");
    let r2 = format!("59-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_60() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-60");
    let r1 = format!("60-r1");
    let r2 = format!("60-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_61() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-61");
    let r1 = format!("61-r1");
    let r2 = format!("61-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_62() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-62");
    let r1 = format!("62-r1");
    let r2 = format!("62-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_63() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-63");
    let r1 = format!("63-r1");
    let r2 = format!("63-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_64() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-64");
    let r1 = format!("64-r1");
    let r2 = format!("64-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_65() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-65");
    let r1 = format!("65-r1");
    let r2 = format!("65-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_66() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-66");
    let r1 = format!("66-r1");
    let r2 = format!("66-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_67() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-67");
    let r1 = format!("67-r1");
    let r2 = format!("67-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_68() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-68");
    let r1 = format!("68-r1");
    let r2 = format!("68-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_69() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-69");
    let r1 = format!("69-r1");
    let r2 = format!("69-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_70() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-70");
    let r1 = format!("70-r1");
    let r2 = format!("70-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_71() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-71");
    let r1 = format!("71-r1");
    let r2 = format!("71-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_72() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-72");
    let r1 = format!("72-r1");
    let r2 = format!("72-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_73() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-73");
    let r1 = format!("73-r1");
    let r2 = format!("73-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_74() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-74");
    let r1 = format!("74-r1");
    let r2 = format!("74-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_75() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-75");
    let r1 = format!("75-r1");
    let r2 = format!("75-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_76() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-76");
    let r1 = format!("76-r1");
    let r2 = format!("76-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_77() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-77");
    let r1 = format!("77-r1");
    let r2 = format!("77-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_78() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-78");
    let r1 = format!("78-r1");
    let r2 = format!("78-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_79() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-79");
    let r1 = format!("79-r1");
    let r2 = format!("79-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_80() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-80");
    let r1 = format!("80-r1");
    let r2 = format!("80-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_81() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-81");
    let r1 = format!("81-r1");
    let r2 = format!("81-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_82() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-82");
    let r1 = format!("82-r1");
    let r2 = format!("82-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_83() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-83");
    let r1 = format!("83-r1");
    let r2 = format!("83-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_84() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-84");
    let r1 = format!("84-r1");
    let r2 = format!("84-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_85() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-85");
    let r1 = format!("85-r1");
    let r2 = format!("85-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_86() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-86");
    let r1 = format!("86-r1");
    let r2 = format!("86-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_87() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-87");
    let r1 = format!("87-r1");
    let r2 = format!("87-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_88() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-88");
    let r1 = format!("88-r1");
    let r2 = format!("88-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_89() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-89");
    let r1 = format!("89-r1");
    let r2 = format!("89-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_90() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-90");
    let r1 = format!("90-r1");
    let r2 = format!("90-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_91() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-91");
    let r1 = format!("91-r1");
    let r2 = format!("91-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_92() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-92");
    let r1 = format!("92-r1");
    let r2 = format!("92-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_93() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-93");
    let r1 = format!("93-r1");
    let r2 = format!("93-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_94() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-94");
    let r1 = format!("94-r1");
    let r2 = format!("94-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_95() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-95");
    let r1 = format!("95-r1");
    let r2 = format!("95-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_96() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-96");
    let r1 = format!("96-r1");
    let r2 = format!("96-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_97() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-97");
    let r1 = format!("97-r1");
    let r2 = format!("97-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_98() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-98");
    let r1 = format!("98-r1");
    let r2 = format!("98-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_99() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-99");
    let r1 = format!("99-r1");
    let r2 = format!("99-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_100() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-100");
    let r1 = format!("100-r1");
    let r2 = format!("100-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_101() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-101");
    let r1 = format!("101-r1");
    let r2 = format!("101-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_102() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-102");
    let r1 = format!("102-r1");
    let r2 = format!("102-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_103() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-103");
    let r1 = format!("103-r1");
    let r2 = format!("103-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_104() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-104");
    let r1 = format!("104-r1");
    let r2 = format!("104-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_105() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-105");
    let r1 = format!("105-r1");
    let r2 = format!("105-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_106() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-106");
    let r1 = format!("106-r1");
    let r2 = format!("106-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_107() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-107");
    let r1 = format!("107-r1");
    let r2 = format!("107-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_108() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-108");
    let r1 = format!("108-r1");
    let r2 = format!("108-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_109() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-109");
    let r1 = format!("109-r1");
    let r2 = format!("109-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_110() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-110");
    let r1 = format!("110-r1");
    let r2 = format!("110-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_111() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-111");
    let r1 = format!("111-r1");
    let r2 = format!("111-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_112() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-112");
    let r1 = format!("112-r1");
    let r2 = format!("112-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_113() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-113");
    let r1 = format!("113-r1");
    let r2 = format!("113-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_114() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-114");
    let r1 = format!("114-r1");
    let r2 = format!("114-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_115() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-115");
    let r1 = format!("115-r1");
    let r2 = format!("115-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_116() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-116");
    let r1 = format!("116-r1");
    let r2 = format!("116-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_117() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-117");
    let r1 = format!("117-r1");
    let r2 = format!("117-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_118() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-118");
    let r1 = format!("118-r1");
    let r2 = format!("118-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_119() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-119");
    let r1 = format!("119-r1");
    let r2 = format!("119-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_120() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-120");
    let r1 = format!("120-r1");
    let r2 = format!("120-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_121() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-121");
    let r1 = format!("121-r1");
    let r2 = format!("121-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_122() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-122");
    let r1 = format!("122-r1");
    let r2 = format!("122-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_123() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-123");
    let r1 = format!("123-r1");
    let r2 = format!("123-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_124() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-124");
    let r1 = format!("124-r1");
    let r2 = format!("124-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_125() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-125");
    let r1 = format!("125-r1");
    let r2 = format!("125-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_126() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-126");
    let r1 = format!("126-r1");
    let r2 = format!("126-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_127() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-127");
    let r1 = format!("127-r1");
    let r2 = format!("127-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_128() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-128");
    let r1 = format!("128-r1");
    let r2 = format!("128-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_129() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-129");
    let r1 = format!("129-r1");
    let r2 = format!("129-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_130() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-130");
    let r1 = format!("130-r1");
    let r2 = format!("130-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_131() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-131");
    let r1 = format!("131-r1");
    let r2 = format!("131-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_132() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-132");
    let r1 = format!("132-r1");
    let r2 = format!("132-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_133() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-133");
    let r1 = format!("133-r1");
    let r2 = format!("133-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_134() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-134");
    let r1 = format!("134-r1");
    let r2 = format!("134-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_135() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-135");
    let r1 = format!("135-r1");
    let r2 = format!("135-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_136() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-136");
    let r1 = format!("136-r1");
    let r2 = format!("136-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_137() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-137");
    let r1 = format!("137-r1");
    let r2 = format!("137-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_138() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-138");
    let r1 = format!("138-r1");
    let r2 = format!("138-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_139() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-139");
    let r1 = format!("139-r1");
    let r2 = format!("139-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn queue_matrix_case_140() {
    let queue = RunQueue::new(RunQueueConfig {
        max_depth_per_session: 3,
        default_timeout: Duration::from_secs(5),
    });

    let sid = format!("session-140");
    let r1 = format!("140-r1");
    let r2 = format!("140-r2");

    queue.enqueue(&sid, &r1, Some(Duration::from_secs(5))).await.unwrap();
    queue.enqueue(&sid, &r2, Some(Duration::from_secs(5))).await.unwrap();

    queue.wait_turn(&sid, &r1, Duration::from_secs(1)).await.unwrap();

    let q2 = queue.clone();
    let sid2 = sid.clone();
    let r2c = r2.clone();
    let waiter = tokio::spawn(async move {
        q2.wait_turn(&sid2, &r2c, Duration::from_secs(2)).await
    });

    tokio::time::sleep(Duration::from_millis(5)).await;
    queue.complete(&sid, &r1, RunStatus::Completed, None).await;
    waiter.await.unwrap().unwrap();

    queue.complete(&sid, &r2, RunStatus::Completed, None).await;
    let rows = queue.list_session_runs(&sid).await;
    assert_eq!(rows.len(), 2);
}
