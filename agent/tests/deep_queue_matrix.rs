use std::time::Duration;

use magicmerlin_agent::{MessageQueue, QueuedMessage};

#[derive(Debug, Clone)]
struct QueueScenario {
    name: &'static str,
    priority: u8,
    session: &'static str,
    text: &'static str,
}

#[tokio::test]
async fn queue_matrix_behaviors() {
    let queue = MessageQueue::new(6);

    let scenarios = vec![
        QueueScenario {
            name: "case_1",
            priority: 60,
            session: "s2",
            text: "message-1",
        },
        QueueScenario {
            name: "case_2",
            priority: 120,
            session: "s3",
            text: "message-2",
        },
        QueueScenario {
            name: "case_3",
            priority: 180,
            session: "s4",
            text: "message-3",
        },
        QueueScenario {
            name: "case_4",
            priority: 240,
            session: "s1",
            text: "message-4",
        },
        QueueScenario {
            name: "case_5",
            priority: 0,
            session: "s2",
            text: "message-5",
        },
        QueueScenario {
            name: "case_6",
            priority: 60,
            session: "s3",
            text: "message-6",
        },
        QueueScenario {
            name: "case_7",
            priority: 120,
            session: "s4",
            text: "message-7",
        },
        QueueScenario {
            name: "case_8",
            priority: 180,
            session: "s1",
            text: "message-8",
        },
        QueueScenario {
            name: "case_9",
            priority: 240,
            session: "s2",
            text: "message-9",
        },
        QueueScenario {
            name: "case_10",
            priority: 0,
            session: "s3",
            text: "message-10",
        },
        QueueScenario {
            name: "case_11",
            priority: 60,
            session: "s4",
            text: "message-11",
        },
        QueueScenario {
            name: "case_12",
            priority: 120,
            session: "s1",
            text: "message-12",
        },
        QueueScenario {
            name: "case_13",
            priority: 180,
            session: "s2",
            text: "message-13",
        },
        QueueScenario {
            name: "case_14",
            priority: 240,
            session: "s3",
            text: "message-14",
        },
        QueueScenario {
            name: "case_15",
            priority: 0,
            session: "s4",
            text: "message-15",
        },
        QueueScenario {
            name: "case_16",
            priority: 60,
            session: "s1",
            text: "message-16",
        },
        QueueScenario {
            name: "case_17",
            priority: 120,
            session: "s2",
            text: "message-17",
        },
        QueueScenario {
            name: "case_18",
            priority: 180,
            session: "s3",
            text: "message-18",
        },
        QueueScenario {
            name: "case_19",
            priority: 240,
            session: "s4",
            text: "message-19",
        },
        QueueScenario {
            name: "case_20",
            priority: 0,
            session: "s1",
            text: "message-20",
        },
        QueueScenario {
            name: "case_21",
            priority: 60,
            session: "s2",
            text: "message-21",
        },
        QueueScenario {
            name: "case_22",
            priority: 120,
            session: "s3",
            text: "message-22",
        },
        QueueScenario {
            name: "case_23",
            priority: 180,
            session: "s4",
            text: "message-23",
        },
        QueueScenario {
            name: "case_24",
            priority: 240,
            session: "s1",
            text: "message-24",
        },
        QueueScenario {
            name: "case_25",
            priority: 0,
            session: "s2",
            text: "message-25",
        },
        QueueScenario {
            name: "case_26",
            priority: 60,
            session: "s3",
            text: "message-26",
        },
        QueueScenario {
            name: "case_27",
            priority: 120,
            session: "s4",
            text: "message-27",
        },
        QueueScenario {
            name: "case_28",
            priority: 180,
            session: "s1",
            text: "message-28",
        },
        QueueScenario {
            name: "case_29",
            priority: 240,
            session: "s2",
            text: "message-29",
        },
        QueueScenario {
            name: "case_30",
            priority: 0,
            session: "s3",
            text: "message-30",
        },
        QueueScenario {
            name: "case_31",
            priority: 60,
            session: "s4",
            text: "message-31",
        },
        QueueScenario {
            name: "case_32",
            priority: 120,
            session: "s1",
            text: "message-32",
        },
        QueueScenario {
            name: "case_33",
            priority: 180,
            session: "s2",
            text: "message-33",
        },
        QueueScenario {
            name: "case_34",
            priority: 240,
            session: "s3",
            text: "message-34",
        },
        QueueScenario {
            name: "case_35",
            priority: 0,
            session: "s4",
            text: "message-35",
        },
        QueueScenario {
            name: "case_36",
            priority: 60,
            session: "s1",
            text: "message-36",
        },
        QueueScenario {
            name: "case_37",
            priority: 120,
            session: "s2",
            text: "message-37",
        },
        QueueScenario {
            name: "case_38",
            priority: 180,
            session: "s3",
            text: "message-38",
        },
        QueueScenario {
            name: "case_39",
            priority: 240,
            session: "s4",
            text: "message-39",
        },
        QueueScenario {
            name: "case_40",
            priority: 0,
            session: "s1",
            text: "message-40",
        },
        QueueScenario {
            name: "case_41",
            priority: 60,
            session: "s2",
            text: "message-41",
        },
        QueueScenario {
            name: "case_42",
            priority: 120,
            session: "s3",
            text: "message-42",
        },
        QueueScenario {
            name: "case_43",
            priority: 180,
            session: "s4",
            text: "message-43",
        },
        QueueScenario {
            name: "case_44",
            priority: 240,
            session: "s1",
            text: "message-44",
        },
        QueueScenario {
            name: "case_45",
            priority: 0,
            session: "s2",
            text: "message-45",
        },
        QueueScenario {
            name: "case_46",
            priority: 60,
            session: "s3",
            text: "message-46",
        },
        QueueScenario {
            name: "case_47",
            priority: 120,
            session: "s4",
            text: "message-47",
        },
        QueueScenario {
            name: "case_48",
            priority: 180,
            session: "s1",
            text: "message-48",
        },
        QueueScenario {
            name: "case_49",
            priority: 240,
            session: "s2",
            text: "message-49",
        },
        QueueScenario {
            name: "case_50",
            priority: 0,
            session: "s3",
            text: "message-50",
        },
        QueueScenario {
            name: "case_51",
            priority: 60,
            session: "s4",
            text: "message-51",
        },
        QueueScenario {
            name: "case_52",
            priority: 120,
            session: "s1",
            text: "message-52",
        },
        QueueScenario {
            name: "case_53",
            priority: 180,
            session: "s2",
            text: "message-53",
        },
        QueueScenario {
            name: "case_54",
            priority: 240,
            session: "s3",
            text: "message-54",
        },
        QueueScenario {
            name: "case_55",
            priority: 0,
            session: "s4",
            text: "message-55",
        },
        QueueScenario {
            name: "case_56",
            priority: 60,
            session: "s1",
            text: "message-56",
        },
        QueueScenario {
            name: "case_57",
            priority: 120,
            session: "s2",
            text: "message-57",
        },
        QueueScenario {
            name: "case_58",
            priority: 180,
            session: "s3",
            text: "message-58",
        },
        QueueScenario {
            name: "case_59",
            priority: 240,
            session: "s4",
            text: "message-59",
        },
        QueueScenario {
            name: "case_60",
            priority: 0,
            session: "s1",
            text: "message-60",
        },
        QueueScenario {
            name: "case_61",
            priority: 60,
            session: "s2",
            text: "message-61",
        },
        QueueScenario {
            name: "case_62",
            priority: 120,
            session: "s3",
            text: "message-62",
        },
        QueueScenario {
            name: "case_63",
            priority: 180,
            session: "s4",
            text: "message-63",
        },
        QueueScenario {
            name: "case_64",
            priority: 240,
            session: "s1",
            text: "message-64",
        },
        QueueScenario {
            name: "case_65",
            priority: 0,
            session: "s2",
            text: "message-65",
        },
        QueueScenario {
            name: "case_66",
            priority: 60,
            session: "s3",
            text: "message-66",
        },
        QueueScenario {
            name: "case_67",
            priority: 120,
            session: "s4",
            text: "message-67",
        },
        QueueScenario {
            name: "case_68",
            priority: 180,
            session: "s1",
            text: "message-68",
        },
        QueueScenario {
            name: "case_69",
            priority: 240,
            session: "s2",
            text: "message-69",
        },
        QueueScenario {
            name: "case_70",
            priority: 0,
            session: "s3",
            text: "message-70",
        },
        QueueScenario {
            name: "case_71",
            priority: 60,
            session: "s4",
            text: "message-71",
        },
        QueueScenario {
            name: "case_72",
            priority: 120,
            session: "s1",
            text: "message-72",
        },
        QueueScenario {
            name: "case_73",
            priority: 180,
            session: "s2",
            text: "message-73",
        },
        QueueScenario {
            name: "case_74",
            priority: 240,
            session: "s3",
            text: "message-74",
        },
        QueueScenario {
            name: "case_75",
            priority: 0,
            session: "s4",
            text: "message-75",
        },
        QueueScenario {
            name: "case_76",
            priority: 60,
            session: "s1",
            text: "message-76",
        },
        QueueScenario {
            name: "case_77",
            priority: 120,
            session: "s2",
            text: "message-77",
        },
        QueueScenario {
            name: "case_78",
            priority: 180,
            session: "s3",
            text: "message-78",
        },
        QueueScenario {
            name: "case_79",
            priority: 240,
            session: "s4",
            text: "message-79",
        },
        QueueScenario {
            name: "case_80",
            priority: 0,
            session: "s1",
            text: "message-80",
        },
        QueueScenario {
            name: "case_81",
            priority: 60,
            session: "s2",
            text: "message-81",
        },
        QueueScenario {
            name: "case_82",
            priority: 120,
            session: "s3",
            text: "message-82",
        },
        QueueScenario {
            name: "case_83",
            priority: 180,
            session: "s4",
            text: "message-83",
        },
        QueueScenario {
            name: "case_84",
            priority: 240,
            session: "s1",
            text: "message-84",
        },
        QueueScenario {
            name: "case_85",
            priority: 0,
            session: "s2",
            text: "message-85",
        },
        QueueScenario {
            name: "case_86",
            priority: 60,
            session: "s3",
            text: "message-86",
        },
        QueueScenario {
            name: "case_87",
            priority: 120,
            session: "s4",
            text: "message-87",
        },
        QueueScenario {
            name: "case_88",
            priority: 180,
            session: "s1",
            text: "message-88",
        },
        QueueScenario {
            name: "case_89",
            priority: 240,
            session: "s2",
            text: "message-89",
        },
        QueueScenario {
            name: "case_90",
            priority: 0,
            session: "s3",
            text: "message-90",
        },
        QueueScenario {
            name: "case_91",
            priority: 60,
            session: "s4",
            text: "message-91",
        },
        QueueScenario {
            name: "case_92",
            priority: 120,
            session: "s1",
            text: "message-92",
        },
        QueueScenario {
            name: "case_93",
            priority: 180,
            session: "s2",
            text: "message-93",
        },
        QueueScenario {
            name: "case_94",
            priority: 240,
            session: "s3",
            text: "message-94",
        },
        QueueScenario {
            name: "case_95",
            priority: 0,
            session: "s4",
            text: "message-95",
        },
        QueueScenario {
            name: "case_96",
            priority: 60,
            session: "s1",
            text: "message-96",
        },
        QueueScenario {
            name: "case_97",
            priority: 120,
            session: "s2",
            text: "message-97",
        },
        QueueScenario {
            name: "case_98",
            priority: 180,
            session: "s3",
            text: "message-98",
        },
        QueueScenario {
            name: "case_99",
            priority: 240,
            session: "s4",
            text: "message-99",
        },
        QueueScenario {
            name: "case_100",
            priority: 0,
            session: "s1",
            text: "message-100",
        },
        QueueScenario {
            name: "case_101",
            priority: 60,
            session: "s2",
            text: "message-101",
        },
        QueueScenario {
            name: "case_102",
            priority: 120,
            session: "s3",
            text: "message-102",
        },
        QueueScenario {
            name: "case_103",
            priority: 180,
            session: "s4",
            text: "message-103",
        },
        QueueScenario {
            name: "case_104",
            priority: 240,
            session: "s1",
            text: "message-104",
        },
        QueueScenario {
            name: "case_105",
            priority: 0,
            session: "s2",
            text: "message-105",
        },
        QueueScenario {
            name: "case_106",
            priority: 60,
            session: "s3",
            text: "message-106",
        },
        QueueScenario {
            name: "case_107",
            priority: 120,
            session: "s4",
            text: "message-107",
        },
        QueueScenario {
            name: "case_108",
            priority: 180,
            session: "s1",
            text: "message-108",
        },
        QueueScenario {
            name: "case_109",
            priority: 240,
            session: "s2",
            text: "message-109",
        },
        QueueScenario {
            name: "case_110",
            priority: 0,
            session: "s3",
            text: "message-110",
        },
        QueueScenario {
            name: "case_111",
            priority: 60,
            session: "s4",
            text: "message-111",
        },
        QueueScenario {
            name: "case_112",
            priority: 120,
            session: "s1",
            text: "message-112",
        },
        QueueScenario {
            name: "case_113",
            priority: 180,
            session: "s2",
            text: "message-113",
        },
        QueueScenario {
            name: "case_114",
            priority: 240,
            session: "s3",
            text: "message-114",
        },
        QueueScenario {
            name: "case_115",
            priority: 0,
            session: "s4",
            text: "message-115",
        },
        QueueScenario {
            name: "case_116",
            priority: 60,
            session: "s1",
            text: "message-116",
        },
        QueueScenario {
            name: "case_117",
            priority: 120,
            session: "s2",
            text: "message-117",
        },
        QueueScenario {
            name: "case_118",
            priority: 180,
            session: "s3",
            text: "message-118",
        },
        QueueScenario {
            name: "case_119",
            priority: 240,
            session: "s4",
            text: "message-119",
        },
        QueueScenario {
            name: "case_120",
            priority: 0,
            session: "s1",
            text: "message-120",
        },
        QueueScenario {
            name: "case_121",
            priority: 60,
            session: "s2",
            text: "message-121",
        },
        QueueScenario {
            name: "case_122",
            priority: 120,
            session: "s3",
            text: "message-122",
        },
        QueueScenario {
            name: "case_123",
            priority: 180,
            session: "s4",
            text: "message-123",
        },
        QueueScenario {
            name: "case_124",
            priority: 240,
            session: "s1",
            text: "message-124",
        },
        QueueScenario {
            name: "case_125",
            priority: 0,
            session: "s2",
            text: "message-125",
        },
        QueueScenario {
            name: "case_126",
            priority: 60,
            session: "s3",
            text: "message-126",
        },
        QueueScenario {
            name: "case_127",
            priority: 120,
            session: "s4",
            text: "message-127",
        },
        QueueScenario {
            name: "case_128",
            priority: 180,
            session: "s1",
            text: "message-128",
        },
        QueueScenario {
            name: "case_129",
            priority: 240,
            session: "s2",
            text: "message-129",
        },
        QueueScenario {
            name: "case_130",
            priority: 0,
            session: "s3",
            text: "message-130",
        },
    ];

    for scenario in scenarios.iter() {
        let _scenario_name = scenario.name;
        let accepted = queue
            .push(QueuedMessage {
                text: scenario.text.to_string(),
                priority: scenario.priority,
                session_key: Some(scenario.session.to_string()),
                created_at: chrono::Utc::now().timestamp(),
            })
            .await
            .expect("push");
        let _ = accepted;
    }

    let mut batches = 0usize;
    while !queue.is_empty().await {
        let batch = queue
            .collect_batch(Duration::from_millis(20))
            .await
            .expect("batch");
        assert!(!batch.is_empty());
        batches += 1;
        if batches > 200 {
            break;
        }
    }

    let stats = queue.stats().await;
    assert!(stats.batches_served >= 1);
}

#[tokio::test]
async fn queue_persistence_large_matrix() {
    let temp = tempfile::tempdir().expect("tmp");
    let path = temp.path().join("queue.json");

    let queue = MessageQueue::new_persistent(32, &path)
        .await
        .expect("queue");

    for i in 0..180 {
        let accepted = queue
            .push(QueuedMessage {
                text: format!("persist-{i}"),
                priority: (i % 255) as u8,
                session_key: Some(format!("s-{}", i % 7)),
                created_at: chrono::Utc::now().timestamp(),
            })
            .await
            .expect("push");
        if i < 32 {
            assert!(accepted);
        }
    }

    let restored = MessageQueue::new_persistent(32, &path)
        .await
        .expect("restored");
    let len = restored.len().await;
    assert!(len > 0);

    let mut drain_count = 0usize;
    loop {
        let popped = restored.pop_now().await.expect("pop");
        if popped.is_none() {
            break;
        }
        drain_count += 1;
        if drain_count > 500 {
            break;
        }
    }

    assert!(drain_count > 0);
}
