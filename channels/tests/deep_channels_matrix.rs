use magicmerlin_channels::framework::{split_text_by_limit, Platform};

#[cfg(feature = "telegram")]
use magicmerlin_channels::telegram::TelegramChannel;

#[test]
fn split_text_by_limit_smoke() {
    let chunks = split_text_by_limit("hello world", 5);
    assert_eq!(chunks.len(), 2);
}

#[test]
fn platform_debug_smoke() {
    assert_eq!(format!("{:?}", Platform::Telegram), "Telegram");
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_smoke() {
    let escaped = TelegramChannel::escape_markdown_v2("hello_world");
    assert!(escaped.contains("\\_"));
}

#[test]
fn framework_split_case_1() {
    let base = "lorem ipsum dolor sit amet 1".repeat(20);
    let limit = (1%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_2() {
    let base = "lorem ipsum dolor sit amet 2".repeat(20);
    let limit = (2%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_3() {
    let base = "lorem ipsum dolor sit amet 3".repeat(20);
    let limit = (3%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_4() {
    let base = "lorem ipsum dolor sit amet 4".repeat(20);
    let limit = (4%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_5() {
    let base = "lorem ipsum dolor sit amet 5".repeat(20);
    let limit = (5%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_6() {
    let base = "lorem ipsum dolor sit amet 6".repeat(20);
    let limit = (6%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_7() {
    let base = "lorem ipsum dolor sit amet 7".repeat(20);
    let limit = (7%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_8() {
    let base = "lorem ipsum dolor sit amet 8".repeat(20);
    let limit = (8%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_9() {
    let base = "lorem ipsum dolor sit amet 9".repeat(20);
    let limit = (9%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_10() {
    let base = "lorem ipsum dolor sit amet 10".repeat(20);
    let limit = (10%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_11() {
    let base = "lorem ipsum dolor sit amet 11".repeat(20);
    let limit = (11%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_12() {
    let base = "lorem ipsum dolor sit amet 12".repeat(20);
    let limit = (12%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_13() {
    let base = "lorem ipsum dolor sit amet 13".repeat(20);
    let limit = (13%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_14() {
    let base = "lorem ipsum dolor sit amet 14".repeat(20);
    let limit = (14%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_15() {
    let base = "lorem ipsum dolor sit amet 15".repeat(20);
    let limit = (15%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_16() {
    let base = "lorem ipsum dolor sit amet 16".repeat(20);
    let limit = (16%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_17() {
    let base = "lorem ipsum dolor sit amet 17".repeat(20);
    let limit = (17%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_18() {
    let base = "lorem ipsum dolor sit amet 18".repeat(20);
    let limit = (18%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_19() {
    let base = "lorem ipsum dolor sit amet 19".repeat(20);
    let limit = (19%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_20() {
    let base = "lorem ipsum dolor sit amet 20".repeat(20);
    let limit = (20%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_21() {
    let base = "lorem ipsum dolor sit amet 21".repeat(20);
    let limit = (21%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_22() {
    let base = "lorem ipsum dolor sit amet 22".repeat(20);
    let limit = (22%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_23() {
    let base = "lorem ipsum dolor sit amet 23".repeat(20);
    let limit = (23%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_24() {
    let base = "lorem ipsum dolor sit amet 24".repeat(20);
    let limit = (24%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_25() {
    let base = "lorem ipsum dolor sit amet 25".repeat(20);
    let limit = (25%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_26() {
    let base = "lorem ipsum dolor sit amet 26".repeat(20);
    let limit = (26%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_27() {
    let base = "lorem ipsum dolor sit amet 27".repeat(20);
    let limit = (27%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_28() {
    let base = "lorem ipsum dolor sit amet 28".repeat(20);
    let limit = (28%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_29() {
    let base = "lorem ipsum dolor sit amet 29".repeat(20);
    let limit = (29%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_30() {
    let base = "lorem ipsum dolor sit amet 30".repeat(20);
    let limit = (30%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_31() {
    let base = "lorem ipsum dolor sit amet 31".repeat(20);
    let limit = (31%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_32() {
    let base = "lorem ipsum dolor sit amet 32".repeat(20);
    let limit = (32%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_33() {
    let base = "lorem ipsum dolor sit amet 33".repeat(20);
    let limit = (33%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_34() {
    let base = "lorem ipsum dolor sit amet 34".repeat(20);
    let limit = (34%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_35() {
    let base = "lorem ipsum dolor sit amet 35".repeat(20);
    let limit = (35%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_36() {
    let base = "lorem ipsum dolor sit amet 36".repeat(20);
    let limit = (36%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_37() {
    let base = "lorem ipsum dolor sit amet 37".repeat(20);
    let limit = (37%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_38() {
    let base = "lorem ipsum dolor sit amet 38".repeat(20);
    let limit = (38%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_39() {
    let base = "lorem ipsum dolor sit amet 39".repeat(20);
    let limit = (39%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_40() {
    let base = "lorem ipsum dolor sit amet 40".repeat(20);
    let limit = (40%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_41() {
    let base = "lorem ipsum dolor sit amet 41".repeat(20);
    let limit = (41%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_42() {
    let base = "lorem ipsum dolor sit amet 42".repeat(20);
    let limit = (42%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_43() {
    let base = "lorem ipsum dolor sit amet 43".repeat(20);
    let limit = (43%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_44() {
    let base = "lorem ipsum dolor sit amet 44".repeat(20);
    let limit = (44%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_45() {
    let base = "lorem ipsum dolor sit amet 45".repeat(20);
    let limit = (45%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_46() {
    let base = "lorem ipsum dolor sit amet 46".repeat(20);
    let limit = (46%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_47() {
    let base = "lorem ipsum dolor sit amet 47".repeat(20);
    let limit = (47%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_48() {
    let base = "lorem ipsum dolor sit amet 48".repeat(20);
    let limit = (48%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_49() {
    let base = "lorem ipsum dolor sit amet 49".repeat(20);
    let limit = (49%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_50() {
    let base = "lorem ipsum dolor sit amet 50".repeat(20);
    let limit = (50%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_51() {
    let base = "lorem ipsum dolor sit amet 51".repeat(20);
    let limit = (51%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_52() {
    let base = "lorem ipsum dolor sit amet 52".repeat(20);
    let limit = (52%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_53() {
    let base = "lorem ipsum dolor sit amet 53".repeat(20);
    let limit = (53%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_54() {
    let base = "lorem ipsum dolor sit amet 54".repeat(20);
    let limit = (54%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_55() {
    let base = "lorem ipsum dolor sit amet 55".repeat(20);
    let limit = (55%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_56() {
    let base = "lorem ipsum dolor sit amet 56".repeat(20);
    let limit = (56%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_57() {
    let base = "lorem ipsum dolor sit amet 57".repeat(20);
    let limit = (57%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_58() {
    let base = "lorem ipsum dolor sit amet 58".repeat(20);
    let limit = (58%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_59() {
    let base = "lorem ipsum dolor sit amet 59".repeat(20);
    let limit = (59%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_60() {
    let base = "lorem ipsum dolor sit amet 60".repeat(20);
    let limit = (60%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_61() {
    let base = "lorem ipsum dolor sit amet 61".repeat(20);
    let limit = (61%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_62() {
    let base = "lorem ipsum dolor sit amet 62".repeat(20);
    let limit = (62%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_63() {
    let base = "lorem ipsum dolor sit amet 63".repeat(20);
    let limit = (63%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_64() {
    let base = "lorem ipsum dolor sit amet 64".repeat(20);
    let limit = (64%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_65() {
    let base = "lorem ipsum dolor sit amet 65".repeat(20);
    let limit = (65%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_66() {
    let base = "lorem ipsum dolor sit amet 66".repeat(20);
    let limit = (66%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_67() {
    let base = "lorem ipsum dolor sit amet 67".repeat(20);
    let limit = (67%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_68() {
    let base = "lorem ipsum dolor sit amet 68".repeat(20);
    let limit = (68%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_69() {
    let base = "lorem ipsum dolor sit amet 69".repeat(20);
    let limit = (69%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_70() {
    let base = "lorem ipsum dolor sit amet 70".repeat(20);
    let limit = (70%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_71() {
    let base = "lorem ipsum dolor sit amet 71".repeat(20);
    let limit = (71%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_72() {
    let base = "lorem ipsum dolor sit amet 72".repeat(20);
    let limit = (72%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_73() {
    let base = "lorem ipsum dolor sit amet 73".repeat(20);
    let limit = (73%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_74() {
    let base = "lorem ipsum dolor sit amet 74".repeat(20);
    let limit = (74%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_75() {
    let base = "lorem ipsum dolor sit amet 75".repeat(20);
    let limit = (75%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_76() {
    let base = "lorem ipsum dolor sit amet 76".repeat(20);
    let limit = (76%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_77() {
    let base = "lorem ipsum dolor sit amet 77".repeat(20);
    let limit = (77%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_78() {
    let base = "lorem ipsum dolor sit amet 78".repeat(20);
    let limit = (78%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_79() {
    let base = "lorem ipsum dolor sit amet 79".repeat(20);
    let limit = (79%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_80() {
    let base = "lorem ipsum dolor sit amet 80".repeat(20);
    let limit = (80%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_81() {
    let base = "lorem ipsum dolor sit amet 81".repeat(20);
    let limit = (81%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_82() {
    let base = "lorem ipsum dolor sit amet 82".repeat(20);
    let limit = (82%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_83() {
    let base = "lorem ipsum dolor sit amet 83".repeat(20);
    let limit = (83%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_84() {
    let base = "lorem ipsum dolor sit amet 84".repeat(20);
    let limit = (84%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_85() {
    let base = "lorem ipsum dolor sit amet 85".repeat(20);
    let limit = (85%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_86() {
    let base = "lorem ipsum dolor sit amet 86".repeat(20);
    let limit = (86%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_87() {
    let base = "lorem ipsum dolor sit amet 87".repeat(20);
    let limit = (87%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_88() {
    let base = "lorem ipsum dolor sit amet 88".repeat(20);
    let limit = (88%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_89() {
    let base = "lorem ipsum dolor sit amet 89".repeat(20);
    let limit = (89%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_90() {
    let base = "lorem ipsum dolor sit amet 90".repeat(20);
    let limit = (90%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_91() {
    let base = "lorem ipsum dolor sit amet 91".repeat(20);
    let limit = (91%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_92() {
    let base = "lorem ipsum dolor sit amet 92".repeat(20);
    let limit = (92%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_93() {
    let base = "lorem ipsum dolor sit amet 93".repeat(20);
    let limit = (93%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_94() {
    let base = "lorem ipsum dolor sit amet 94".repeat(20);
    let limit = (94%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_95() {
    let base = "lorem ipsum dolor sit amet 95".repeat(20);
    let limit = (95%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_96() {
    let base = "lorem ipsum dolor sit amet 96".repeat(20);
    let limit = (96%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_97() {
    let base = "lorem ipsum dolor sit amet 97".repeat(20);
    let limit = (97%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_98() {
    let base = "lorem ipsum dolor sit amet 98".repeat(20);
    let limit = (98%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_99() {
    let base = "lorem ipsum dolor sit amet 99".repeat(20);
    let limit = (99%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_100() {
    let base = "lorem ipsum dolor sit amet 100".repeat(20);
    let limit = (100%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_101() {
    let base = "lorem ipsum dolor sit amet 101".repeat(20);
    let limit = (101%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_102() {
    let base = "lorem ipsum dolor sit amet 102".repeat(20);
    let limit = (102%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_103() {
    let base = "lorem ipsum dolor sit amet 103".repeat(20);
    let limit = (103%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_104() {
    let base = "lorem ipsum dolor sit amet 104".repeat(20);
    let limit = (104%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_105() {
    let base = "lorem ipsum dolor sit amet 105".repeat(20);
    let limit = (105%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_106() {
    let base = "lorem ipsum dolor sit amet 106".repeat(20);
    let limit = (106%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_107() {
    let base = "lorem ipsum dolor sit amet 107".repeat(20);
    let limit = (107%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_108() {
    let base = "lorem ipsum dolor sit amet 108".repeat(20);
    let limit = (108%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_109() {
    let base = "lorem ipsum dolor sit amet 109".repeat(20);
    let limit = (109%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_110() {
    let base = "lorem ipsum dolor sit amet 110".repeat(20);
    let limit = (110%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_111() {
    let base = "lorem ipsum dolor sit amet 111".repeat(20);
    let limit = (111%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_112() {
    let base = "lorem ipsum dolor sit amet 112".repeat(20);
    let limit = (112%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_113() {
    let base = "lorem ipsum dolor sit amet 113".repeat(20);
    let limit = (113%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_114() {
    let base = "lorem ipsum dolor sit amet 114".repeat(20);
    let limit = (114%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_115() {
    let base = "lorem ipsum dolor sit amet 115".repeat(20);
    let limit = (115%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_116() {
    let base = "lorem ipsum dolor sit amet 116".repeat(20);
    let limit = (116%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_117() {
    let base = "lorem ipsum dolor sit amet 117".repeat(20);
    let limit = (117%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_118() {
    let base = "lorem ipsum dolor sit amet 118".repeat(20);
    let limit = (118%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_119() {
    let base = "lorem ipsum dolor sit amet 119".repeat(20);
    let limit = (119%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_120() {
    let base = "lorem ipsum dolor sit amet 120".repeat(20);
    let limit = (120%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_121() {
    let base = "lorem ipsum dolor sit amet 121".repeat(20);
    let limit = (121%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_122() {
    let base = "lorem ipsum dolor sit amet 122".repeat(20);
    let limit = (122%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_123() {
    let base = "lorem ipsum dolor sit amet 123".repeat(20);
    let limit = (123%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_124() {
    let base = "lorem ipsum dolor sit amet 124".repeat(20);
    let limit = (124%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_125() {
    let base = "lorem ipsum dolor sit amet 125".repeat(20);
    let limit = (125%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_126() {
    let base = "lorem ipsum dolor sit amet 126".repeat(20);
    let limit = (126%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_127() {
    let base = "lorem ipsum dolor sit amet 127".repeat(20);
    let limit = (127%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_128() {
    let base = "lorem ipsum dolor sit amet 128".repeat(20);
    let limit = (128%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_129() {
    let base = "lorem ipsum dolor sit amet 129".repeat(20);
    let limit = (129%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_130() {
    let base = "lorem ipsum dolor sit amet 130".repeat(20);
    let limit = (130%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_131() {
    let base = "lorem ipsum dolor sit amet 131".repeat(20);
    let limit = (131%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_132() {
    let base = "lorem ipsum dolor sit amet 132".repeat(20);
    let limit = (132%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_133() {
    let base = "lorem ipsum dolor sit amet 133".repeat(20);
    let limit = (133%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_134() {
    let base = "lorem ipsum dolor sit amet 134".repeat(20);
    let limit = (134%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_135() {
    let base = "lorem ipsum dolor sit amet 135".repeat(20);
    let limit = (135%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_136() {
    let base = "lorem ipsum dolor sit amet 136".repeat(20);
    let limit = (136%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_137() {
    let base = "lorem ipsum dolor sit amet 137".repeat(20);
    let limit = (137%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_138() {
    let base = "lorem ipsum dolor sit amet 138".repeat(20);
    let limit = (138%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_139() {
    let base = "lorem ipsum dolor sit amet 139".repeat(20);
    let limit = (139%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_140() {
    let base = "lorem ipsum dolor sit amet 140".repeat(20);
    let limit = (140%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_141() {
    let base = "lorem ipsum dolor sit amet 141".repeat(20);
    let limit = (141%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_142() {
    let base = "lorem ipsum dolor sit amet 142".repeat(20);
    let limit = (142%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_143() {
    let base = "lorem ipsum dolor sit amet 143".repeat(20);
    let limit = (143%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_144() {
    let base = "lorem ipsum dolor sit amet 144".repeat(20);
    let limit = (144%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_145() {
    let base = "lorem ipsum dolor sit amet 145".repeat(20);
    let limit = (145%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_146() {
    let base = "lorem ipsum dolor sit amet 146".repeat(20);
    let limit = (146%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_147() {
    let base = "lorem ipsum dolor sit amet 147".repeat(20);
    let limit = (147%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_148() {
    let base = "lorem ipsum dolor sit amet 148".repeat(20);
    let limit = (148%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_149() {
    let base = "lorem ipsum dolor sit amet 149".repeat(20);
    let limit = (149%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_150() {
    let base = "lorem ipsum dolor sit amet 150".repeat(20);
    let limit = (150%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_151() {
    let base = "lorem ipsum dolor sit amet 151".repeat(20);
    let limit = (151%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_152() {
    let base = "lorem ipsum dolor sit amet 152".repeat(20);
    let limit = (152%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_153() {
    let base = "lorem ipsum dolor sit amet 153".repeat(20);
    let limit = (153%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_154() {
    let base = "lorem ipsum dolor sit amet 154".repeat(20);
    let limit = (154%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_155() {
    let base = "lorem ipsum dolor sit amet 155".repeat(20);
    let limit = (155%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_156() {
    let base = "lorem ipsum dolor sit amet 156".repeat(20);
    let limit = (156%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_157() {
    let base = "lorem ipsum dolor sit amet 157".repeat(20);
    let limit = (157%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_158() {
    let base = "lorem ipsum dolor sit amet 158".repeat(20);
    let limit = (158%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_159() {
    let base = "lorem ipsum dolor sit amet 159".repeat(20);
    let limit = (159%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_160() {
    let base = "lorem ipsum dolor sit amet 160".repeat(20);
    let limit = (160%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_161() {
    let base = "lorem ipsum dolor sit amet 161".repeat(20);
    let limit = (161%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_162() {
    let base = "lorem ipsum dolor sit amet 162".repeat(20);
    let limit = (162%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_163() {
    let base = "lorem ipsum dolor sit amet 163".repeat(20);
    let limit = (163%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_164() {
    let base = "lorem ipsum dolor sit amet 164".repeat(20);
    let limit = (164%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_165() {
    let base = "lorem ipsum dolor sit amet 165".repeat(20);
    let limit = (165%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_166() {
    let base = "lorem ipsum dolor sit amet 166".repeat(20);
    let limit = (166%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_167() {
    let base = "lorem ipsum dolor sit amet 167".repeat(20);
    let limit = (167%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_168() {
    let base = "lorem ipsum dolor sit amet 168".repeat(20);
    let limit = (168%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_169() {
    let base = "lorem ipsum dolor sit amet 169".repeat(20);
    let limit = (169%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_170() {
    let base = "lorem ipsum dolor sit amet 170".repeat(20);
    let limit = (170%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_171() {
    let base = "lorem ipsum dolor sit amet 171".repeat(20);
    let limit = (171%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_172() {
    let base = "lorem ipsum dolor sit amet 172".repeat(20);
    let limit = (172%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_173() {
    let base = "lorem ipsum dolor sit amet 173".repeat(20);
    let limit = (173%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_174() {
    let base = "lorem ipsum dolor sit amet 174".repeat(20);
    let limit = (174%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_175() {
    let base = "lorem ipsum dolor sit amet 175".repeat(20);
    let limit = (175%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_176() {
    let base = "lorem ipsum dolor sit amet 176".repeat(20);
    let limit = (176%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_177() {
    let base = "lorem ipsum dolor sit amet 177".repeat(20);
    let limit = (177%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_178() {
    let base = "lorem ipsum dolor sit amet 178".repeat(20);
    let limit = (178%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_179() {
    let base = "lorem ipsum dolor sit amet 179".repeat(20);
    let limit = (179%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_180() {
    let base = "lorem ipsum dolor sit amet 180".repeat(20);
    let limit = (180%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_181() {
    let base = "lorem ipsum dolor sit amet 181".repeat(20);
    let limit = (181%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_182() {
    let base = "lorem ipsum dolor sit amet 182".repeat(20);
    let limit = (182%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_183() {
    let base = "lorem ipsum dolor sit amet 183".repeat(20);
    let limit = (183%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_184() {
    let base = "lorem ipsum dolor sit amet 184".repeat(20);
    let limit = (184%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_185() {
    let base = "lorem ipsum dolor sit amet 185".repeat(20);
    let limit = (185%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_186() {
    let base = "lorem ipsum dolor sit amet 186".repeat(20);
    let limit = (186%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_187() {
    let base = "lorem ipsum dolor sit amet 187".repeat(20);
    let limit = (187%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_188() {
    let base = "lorem ipsum dolor sit amet 188".repeat(20);
    let limit = (188%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_189() {
    let base = "lorem ipsum dolor sit amet 189".repeat(20);
    let limit = (189%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_190() {
    let base = "lorem ipsum dolor sit amet 190".repeat(20);
    let limit = (190%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_191() {
    let base = "lorem ipsum dolor sit amet 191".repeat(20);
    let limit = (191%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_192() {
    let base = "lorem ipsum dolor sit amet 192".repeat(20);
    let limit = (192%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_193() {
    let base = "lorem ipsum dolor sit amet 193".repeat(20);
    let limit = (193%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_194() {
    let base = "lorem ipsum dolor sit amet 194".repeat(20);
    let limit = (194%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_195() {
    let base = "lorem ipsum dolor sit amet 195".repeat(20);
    let limit = (195%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_196() {
    let base = "lorem ipsum dolor sit amet 196".repeat(20);
    let limit = (196%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_197() {
    let base = "lorem ipsum dolor sit amet 197".repeat(20);
    let limit = (197%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_198() {
    let base = "lorem ipsum dolor sit amet 198".repeat(20);
    let limit = (198%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_199() {
    let base = "lorem ipsum dolor sit amet 199".repeat(20);
    let limit = (199%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_200() {
    let base = "lorem ipsum dolor sit amet 200".repeat(20);
    let limit = (200%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_201() {
    let base = "lorem ipsum dolor sit amet 201".repeat(20);
    let limit = (201%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_202() {
    let base = "lorem ipsum dolor sit amet 202".repeat(20);
    let limit = (202%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_203() {
    let base = "lorem ipsum dolor sit amet 203".repeat(20);
    let limit = (203%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_204() {
    let base = "lorem ipsum dolor sit amet 204".repeat(20);
    let limit = (204%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_205() {
    let base = "lorem ipsum dolor sit amet 205".repeat(20);
    let limit = (205%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_206() {
    let base = "lorem ipsum dolor sit amet 206".repeat(20);
    let limit = (206%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_207() {
    let base = "lorem ipsum dolor sit amet 207".repeat(20);
    let limit = (207%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_208() {
    let base = "lorem ipsum dolor sit amet 208".repeat(20);
    let limit = (208%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_209() {
    let base = "lorem ipsum dolor sit amet 209".repeat(20);
    let limit = (209%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_210() {
    let base = "lorem ipsum dolor sit amet 210".repeat(20);
    let limit = (210%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_211() {
    let base = "lorem ipsum dolor sit amet 211".repeat(20);
    let limit = (211%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_212() {
    let base = "lorem ipsum dolor sit amet 212".repeat(20);
    let limit = (212%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_213() {
    let base = "lorem ipsum dolor sit amet 213".repeat(20);
    let limit = (213%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_214() {
    let base = "lorem ipsum dolor sit amet 214".repeat(20);
    let limit = (214%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_215() {
    let base = "lorem ipsum dolor sit amet 215".repeat(20);
    let limit = (215%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_216() {
    let base = "lorem ipsum dolor sit amet 216".repeat(20);
    let limit = (216%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_217() {
    let base = "lorem ipsum dolor sit amet 217".repeat(20);
    let limit = (217%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_218() {
    let base = "lorem ipsum dolor sit amet 218".repeat(20);
    let limit = (218%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_219() {
    let base = "lorem ipsum dolor sit amet 219".repeat(20);
    let limit = (219%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_220() {
    let base = "lorem ipsum dolor sit amet 220".repeat(20);
    let limit = (220%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_221() {
    let base = "lorem ipsum dolor sit amet 221".repeat(20);
    let limit = (221%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_222() {
    let base = "lorem ipsum dolor sit amet 222".repeat(20);
    let limit = (222%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_223() {
    let base = "lorem ipsum dolor sit amet 223".repeat(20);
    let limit = (223%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_224() {
    let base = "lorem ipsum dolor sit amet 224".repeat(20);
    let limit = (224%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_225() {
    let base = "lorem ipsum dolor sit amet 225".repeat(20);
    let limit = (225%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_226() {
    let base = "lorem ipsum dolor sit amet 226".repeat(20);
    let limit = (226%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_227() {
    let base = "lorem ipsum dolor sit amet 227".repeat(20);
    let limit = (227%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_228() {
    let base = "lorem ipsum dolor sit amet 228".repeat(20);
    let limit = (228%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_229() {
    let base = "lorem ipsum dolor sit amet 229".repeat(20);
    let limit = (229%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_230() {
    let base = "lorem ipsum dolor sit amet 230".repeat(20);
    let limit = (230%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_231() {
    let base = "lorem ipsum dolor sit amet 231".repeat(20);
    let limit = (231%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_232() {
    let base = "lorem ipsum dolor sit amet 232".repeat(20);
    let limit = (232%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_233() {
    let base = "lorem ipsum dolor sit amet 233".repeat(20);
    let limit = (233%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_234() {
    let base = "lorem ipsum dolor sit amet 234".repeat(20);
    let limit = (234%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_235() {
    let base = "lorem ipsum dolor sit amet 235".repeat(20);
    let limit = (235%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_236() {
    let base = "lorem ipsum dolor sit amet 236".repeat(20);
    let limit = (236%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_237() {
    let base = "lorem ipsum dolor sit amet 237".repeat(20);
    let limit = (237%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_238() {
    let base = "lorem ipsum dolor sit amet 238".repeat(20);
    let limit = (238%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_239() {
    let base = "lorem ipsum dolor sit amet 239".repeat(20);
    let limit = (239%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_240() {
    let base = "lorem ipsum dolor sit amet 240".repeat(20);
    let limit = (240%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_241() {
    let base = "lorem ipsum dolor sit amet 241".repeat(20);
    let limit = (241%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_242() {
    let base = "lorem ipsum dolor sit amet 242".repeat(20);
    let limit = (242%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_243() {
    let base = "lorem ipsum dolor sit amet 243".repeat(20);
    let limit = (243%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_244() {
    let base = "lorem ipsum dolor sit amet 244".repeat(20);
    let limit = (244%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_245() {
    let base = "lorem ipsum dolor sit amet 245".repeat(20);
    let limit = (245%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_246() {
    let base = "lorem ipsum dolor sit amet 246".repeat(20);
    let limit = (246%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_247() {
    let base = "lorem ipsum dolor sit amet 247".repeat(20);
    let limit = (247%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_248() {
    let base = "lorem ipsum dolor sit amet 248".repeat(20);
    let limit = (248%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_249() {
    let base = "lorem ipsum dolor sit amet 249".repeat(20);
    let limit = (249%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_250() {
    let base = "lorem ipsum dolor sit amet 250".repeat(20);
    let limit = (250%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_251() {
    let base = "lorem ipsum dolor sit amet 251".repeat(20);
    let limit = (251%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_252() {
    let base = "lorem ipsum dolor sit amet 252".repeat(20);
    let limit = (252%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_253() {
    let base = "lorem ipsum dolor sit amet 253".repeat(20);
    let limit = (253%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_254() {
    let base = "lorem ipsum dolor sit amet 254".repeat(20);
    let limit = (254%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_255() {
    let base = "lorem ipsum dolor sit amet 255".repeat(20);
    let limit = (255%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_256() {
    let base = "lorem ipsum dolor sit amet 256".repeat(20);
    let limit = (256%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_257() {
    let base = "lorem ipsum dolor sit amet 257".repeat(20);
    let limit = (257%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_258() {
    let base = "lorem ipsum dolor sit amet 258".repeat(20);
    let limit = (258%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_259() {
    let base = "lorem ipsum dolor sit amet 259".repeat(20);
    let limit = (259%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_260() {
    let base = "lorem ipsum dolor sit amet 260".repeat(20);
    let limit = (260%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_261() {
    let base = "lorem ipsum dolor sit amet 261".repeat(20);
    let limit = (261%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_262() {
    let base = "lorem ipsum dolor sit amet 262".repeat(20);
    let limit = (262%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_263() {
    let base = "lorem ipsum dolor sit amet 263".repeat(20);
    let limit = (263%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_264() {
    let base = "lorem ipsum dolor sit amet 264".repeat(20);
    let limit = (264%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_265() {
    let base = "lorem ipsum dolor sit amet 265".repeat(20);
    let limit = (265%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_266() {
    let base = "lorem ipsum dolor sit amet 266".repeat(20);
    let limit = (266%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_267() {
    let base = "lorem ipsum dolor sit amet 267".repeat(20);
    let limit = (267%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_268() {
    let base = "lorem ipsum dolor sit amet 268".repeat(20);
    let limit = (268%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_269() {
    let base = "lorem ipsum dolor sit amet 269".repeat(20);
    let limit = (269%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_270() {
    let base = "lorem ipsum dolor sit amet 270".repeat(20);
    let limit = (270%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_271() {
    let base = "lorem ipsum dolor sit amet 271".repeat(20);
    let limit = (271%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_272() {
    let base = "lorem ipsum dolor sit amet 272".repeat(20);
    let limit = (272%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_273() {
    let base = "lorem ipsum dolor sit amet 273".repeat(20);
    let limit = (273%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_274() {
    let base = "lorem ipsum dolor sit amet 274".repeat(20);
    let limit = (274%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_275() {
    let base = "lorem ipsum dolor sit amet 275".repeat(20);
    let limit = (275%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_276() {
    let base = "lorem ipsum dolor sit amet 276".repeat(20);
    let limit = (276%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_277() {
    let base = "lorem ipsum dolor sit amet 277".repeat(20);
    let limit = (277%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_278() {
    let base = "lorem ipsum dolor sit amet 278".repeat(20);
    let limit = (278%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_279() {
    let base = "lorem ipsum dolor sit amet 279".repeat(20);
    let limit = (279%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_280() {
    let base = "lorem ipsum dolor sit amet 280".repeat(20);
    let limit = (280%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_281() {
    let base = "lorem ipsum dolor sit amet 281".repeat(20);
    let limit = (281%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_282() {
    let base = "lorem ipsum dolor sit amet 282".repeat(20);
    let limit = (282%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_283() {
    let base = "lorem ipsum dolor sit amet 283".repeat(20);
    let limit = (283%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_284() {
    let base = "lorem ipsum dolor sit amet 284".repeat(20);
    let limit = (284%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_285() {
    let base = "lorem ipsum dolor sit amet 285".repeat(20);
    let limit = (285%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_286() {
    let base = "lorem ipsum dolor sit amet 286".repeat(20);
    let limit = (286%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_287() {
    let base = "lorem ipsum dolor sit amet 287".repeat(20);
    let limit = (287%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_288() {
    let base = "lorem ipsum dolor sit amet 288".repeat(20);
    let limit = (288%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_289() {
    let base = "lorem ipsum dolor sit amet 289".repeat(20);
    let limit = (289%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_290() {
    let base = "lorem ipsum dolor sit amet 290".repeat(20);
    let limit = (290%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_291() {
    let base = "lorem ipsum dolor sit amet 291".repeat(20);
    let limit = (291%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_292() {
    let base = "lorem ipsum dolor sit amet 292".repeat(20);
    let limit = (292%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_293() {
    let base = "lorem ipsum dolor sit amet 293".repeat(20);
    let limit = (293%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_294() {
    let base = "lorem ipsum dolor sit amet 294".repeat(20);
    let limit = (294%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_295() {
    let base = "lorem ipsum dolor sit amet 295".repeat(20);
    let limit = (295%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_296() {
    let base = "lorem ipsum dolor sit amet 296".repeat(20);
    let limit = (296%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_297() {
    let base = "lorem ipsum dolor sit amet 297".repeat(20);
    let limit = (297%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_298() {
    let base = "lorem ipsum dolor sit amet 298".repeat(20);
    let limit = (298%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_299() {
    let base = "lorem ipsum dolor sit amet 299".repeat(20);
    let limit = (299%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[test]
fn framework_split_case_300() {
    let base = "lorem ipsum dolor sit amet 300".repeat(20);
    let limit = (300%40)+10;
    let chunks = split_text_by_limit(&base, limit);
    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|chunk| chunk.len() <= limit));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_1() {
    let raw = "case_1_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_2() {
    let raw = "case_2_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_3() {
    let raw = "case_3_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_4() {
    let raw = "case_4_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_5() {
    let raw = "case_5_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_6() {
    let raw = "case_6_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_7() {
    let raw = "case_7_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_8() {
    let raw = "case_8_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_9() {
    let raw = "case_9_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_10() {
    let raw = "case_10_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_11() {
    let raw = "case_11_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_12() {
    let raw = "case_12_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_13() {
    let raw = "case_13_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_14() {
    let raw = "case_14_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_15() {
    let raw = "case_15_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_16() {
    let raw = "case_16_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_17() {
    let raw = "case_17_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_18() {
    let raw = "case_18_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_19() {
    let raw = "case_19_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_20() {
    let raw = "case_20_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_21() {
    let raw = "case_21_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_22() {
    let raw = "case_22_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_23() {
    let raw = "case_23_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_24() {
    let raw = "case_24_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_25() {
    let raw = "case_25_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_26() {
    let raw = "case_26_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_27() {
    let raw = "case_27_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_28() {
    let raw = "case_28_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_29() {
    let raw = "case_29_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_30() {
    let raw = "case_30_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_31() {
    let raw = "case_31_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_32() {
    let raw = "case_32_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_33() {
    let raw = "case_33_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_34() {
    let raw = "case_34_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_35() {
    let raw = "case_35_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_36() {
    let raw = "case_36_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_37() {
    let raw = "case_37_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_38() {
    let raw = "case_38_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_39() {
    let raw = "case_39_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_40() {
    let raw = "case_40_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_41() {
    let raw = "case_41_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_42() {
    let raw = "case_42_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_43() {
    let raw = "case_43_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_44() {
    let raw = "case_44_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_45() {
    let raw = "case_45_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_46() {
    let raw = "case_46_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_47() {
    let raw = "case_47_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_48() {
    let raw = "case_48_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_49() {
    let raw = "case_49_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_50() {
    let raw = "case_50_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_51() {
    let raw = "case_51_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_52() {
    let raw = "case_52_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_53() {
    let raw = "case_53_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_54() {
    let raw = "case_54_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_55() {
    let raw = "case_55_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_56() {
    let raw = "case_56_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_57() {
    let raw = "case_57_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_58() {
    let raw = "case_58_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_59() {
    let raw = "case_59_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_60() {
    let raw = "case_60_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_61() {
    let raw = "case_61_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_62() {
    let raw = "case_62_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_63() {
    let raw = "case_63_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_64() {
    let raw = "case_64_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_65() {
    let raw = "case_65_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_66() {
    let raw = "case_66_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_67() {
    let raw = "case_67_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_68() {
    let raw = "case_68_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_69() {
    let raw = "case_69_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_70() {
    let raw = "case_70_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_71() {
    let raw = "case_71_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_72() {
    let raw = "case_72_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_73() {
    let raw = "case_73_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_74() {
    let raw = "case_74_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_75() {
    let raw = "case_75_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_76() {
    let raw = "case_76_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_77() {
    let raw = "case_77_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_78() {
    let raw = "case_78_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_79() {
    let raw = "case_79_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_80() {
    let raw = "case_80_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_81() {
    let raw = "case_81_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_82() {
    let raw = "case_82_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_83() {
    let raw = "case_83_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_84() {
    let raw = "case_84_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_85() {
    let raw = "case_85_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_86() {
    let raw = "case_86_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_87() {
    let raw = "case_87_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_88() {
    let raw = "case_88_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_89() {
    let raw = "case_89_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_90() {
    let raw = "case_90_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_91() {
    let raw = "case_91_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_92() {
    let raw = "case_92_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_93() {
    let raw = "case_93_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_94() {
    let raw = "case_94_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_95() {
    let raw = "case_95_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_96() {
    let raw = "case_96_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_97() {
    let raw = "case_97_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_98() {
    let raw = "case_98_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_99() {
    let raw = "case_99_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_100() {
    let raw = "case_100_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_101() {
    let raw = "case_101_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_102() {
    let raw = "case_102_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_103() {
    let raw = "case_103_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_104() {
    let raw = "case_104_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_105() {
    let raw = "case_105_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_106() {
    let raw = "case_106_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_107() {
    let raw = "case_107_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_108() {
    let raw = "case_108_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_109() {
    let raw = "case_109_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_110() {
    let raw = "case_110_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_111() {
    let raw = "case_111_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_112() {
    let raw = "case_112_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_113() {
    let raw = "case_113_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_114() {
    let raw = "case_114_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_115() {
    let raw = "case_115_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_116() {
    let raw = "case_116_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_117() {
    let raw = "case_117_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_118() {
    let raw = "case_118_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_119() {
    let raw = "case_119_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_120() {
    let raw = "case_120_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_121() {
    let raw = "case_121_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_122() {
    let raw = "case_122_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_123() {
    let raw = "case_123_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_124() {
    let raw = "case_124_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_125() {
    let raw = "case_125_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_126() {
    let raw = "case_126_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_127() {
    let raw = "case_127_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_128() {
    let raw = "case_128_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_129() {
    let raw = "case_129_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_130() {
    let raw = "case_130_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_131() {
    let raw = "case_131_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_132() {
    let raw = "case_132_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_133() {
    let raw = "case_133_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_134() {
    let raw = "case_134_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_135() {
    let raw = "case_135_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_136() {
    let raw = "case_136_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_137() {
    let raw = "case_137_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_138() {
    let raw = "case_138_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_139() {
    let raw = "case_139_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_140() {
    let raw = "case_140_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_141() {
    let raw = "case_141_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_142() {
    let raw = "case_142_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_143() {
    let raw = "case_143_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_144() {
    let raw = "case_144_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_145() {
    let raw = "case_145_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_146() {
    let raw = "case_146_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_147() {
    let raw = "case_147_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_148() {
    let raw = "case_148_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_149() {
    let raw = "case_149_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_150() {
    let raw = "case_150_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_151() {
    let raw = "case_151_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_152() {
    let raw = "case_152_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_153() {
    let raw = "case_153_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_154() {
    let raw = "case_154_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_155() {
    let raw = "case_155_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_156() {
    let raw = "case_156_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_157() {
    let raw = "case_157_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_158() {
    let raw = "case_158_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_159() {
    let raw = "case_159_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_160() {
    let raw = "case_160_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_161() {
    let raw = "case_161_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_162() {
    let raw = "case_162_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_163() {
    let raw = "case_163_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_164() {
    let raw = "case_164_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_165() {
    let raw = "case_165_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_166() {
    let raw = "case_166_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_167() {
    let raw = "case_167_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_168() {
    let raw = "case_168_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_169() {
    let raw = "case_169_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_170() {
    let raw = "case_170_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_171() {
    let raw = "case_171_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_172() {
    let raw = "case_172_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_173() {
    let raw = "case_173_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_174() {
    let raw = "case_174_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_175() {
    let raw = "case_175_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_176() {
    let raw = "case_176_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_177() {
    let raw = "case_177_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_178() {
    let raw = "case_178_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_179() {
    let raw = "case_179_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_180() {
    let raw = "case_180_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_181() {
    let raw = "case_181_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_182() {
    let raw = "case_182_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_183() {
    let raw = "case_183_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_184() {
    let raw = "case_184_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_185() {
    let raw = "case_185_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_186() {
    let raw = "case_186_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_187() {
    let raw = "case_187_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_188() {
    let raw = "case_188_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_189() {
    let raw = "case_189_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_190() {
    let raw = "case_190_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_191() {
    let raw = "case_191_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_192() {
    let raw = "case_192_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_193() {
    let raw = "case_193_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_194() {
    let raw = "case_194_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_195() {
    let raw = "case_195_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_196() {
    let raw = "case_196_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_197() {
    let raw = "case_197_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_198() {
    let raw = "case_198_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_199() {
    let raw = "case_199_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_200() {
    let raw = "case_200_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_201() {
    let raw = "case_201_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_202() {
    let raw = "case_202_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_203() {
    let raw = "case_203_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_204() {
    let raw = "case_204_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_205() {
    let raw = "case_205_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_206() {
    let raw = "case_206_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_207() {
    let raw = "case_207_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_208() {
    let raw = "case_208_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_209() {
    let raw = "case_209_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_210() {
    let raw = "case_210_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_211() {
    let raw = "case_211_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_212() {
    let raw = "case_212_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_213() {
    let raw = "case_213_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_214() {
    let raw = "case_214_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_215() {
    let raw = "case_215_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_216() {
    let raw = "case_216_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_217() {
    let raw = "case_217_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_218() {
    let raw = "case_218_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_219() {
    let raw = "case_219_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_220() {
    let raw = "case_220_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_221() {
    let raw = "case_221_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_222() {
    let raw = "case_222_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_223() {
    let raw = "case_223_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_224() {
    let raw = "case_224_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_225() {
    let raw = "case_225_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_226() {
    let raw = "case_226_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_227() {
    let raw = "case_227_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_228() {
    let raw = "case_228_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_229() {
    let raw = "case_229_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_230() {
    let raw = "case_230_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_231() {
    let raw = "case_231_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_232() {
    let raw = "case_232_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_233() {
    let raw = "case_233_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_234() {
    let raw = "case_234_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_235() {
    let raw = "case_235_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_236() {
    let raw = "case_236_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_237() {
    let raw = "case_237_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_238() {
    let raw = "case_238_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_239() {
    let raw = "case_239_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_240() {
    let raw = "case_240_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_241() {
    let raw = "case_241_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_242() {
    let raw = "case_242_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_243() {
    let raw = "case_243_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_244() {
    let raw = "case_244_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_245() {
    let raw = "case_245_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_246() {
    let raw = "case_246_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_247() {
    let raw = "case_247_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_248() {
    let raw = "case_248_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_249() {
    let raw = "case_249_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_250() {
    let raw = "case_250_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_251() {
    let raw = "case_251_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_252() {
    let raw = "case_252_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_253() {
    let raw = "case_253_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_254() {
    let raw = "case_254_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_255() {
    let raw = "case_255_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_256() {
    let raw = "case_256_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_257() {
    let raw = "case_257_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_258() {
    let raw = "case_258_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_259() {
    let raw = "case_259_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_260() {
    let raw = "case_260_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_261() {
    let raw = "case_261_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_262() {
    let raw = "case_262_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_263() {
    let raw = "case_263_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_264() {
    let raw = "case_264_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_265() {
    let raw = "case_265_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_266() {
    let raw = "case_266_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_267() {
    let raw = "case_267_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_268() {
    let raw = "case_268_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_269() {
    let raw = "case_269_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_270() {
    let raw = "case_270_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_271() {
    let raw = "case_271_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_272() {
    let raw = "case_272_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_273() {
    let raw = "case_273_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_274() {
    let raw = "case_274_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_275() {
    let raw = "case_275_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_276() {
    let raw = "case_276_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_277() {
    let raw = "case_277_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_278() {
    let raw = "case_278_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_279() {
    let raw = "case_279_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_280() {
    let raw = "case_280_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_281() {
    let raw = "case_281_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_282() {
    let raw = "case_282_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_283() {
    let raw = "case_283_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_284() {
    let raw = "case_284_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_285() {
    let raw = "case_285_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_286() {
    let raw = "case_286_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_287() {
    let raw = "case_287_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_288() {
    let raw = "case_288_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_289() {
    let raw = "case_289_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_290() {
    let raw = "case_290_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_291() {
    let raw = "case_291_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_292() {
    let raw = "case_292_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_293() {
    let raw = "case_293_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_294() {
    let raw = "case_294_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_295() {
    let raw = "case_295_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_296() {
    let raw = "case_296_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_297() {
    let raw = "case_297_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_298() {
    let raw = "case_298_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_299() {
    let raw = "case_299_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_escape_case_300() {
    let raw = "case_300_*_[]()~>#+-=|{}.!";
    let escaped = TelegramChannel::escape_markdown_v2(raw);
    assert!(escaped.contains(r"\*"));
    assert!(escaped.contains(r"\["));
    assert!(escaped.contains(r"\]"));
    assert!(escaped.contains(r"\("));
    assert!(escaped.contains(r"\)"));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_1() {
    let text = "x".repeat(4100 + 1);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_2() {
    let text = "x".repeat(4100 + 2);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_3() {
    let text = "x".repeat(4100 + 3);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_4() {
    let text = "x".repeat(4100 + 4);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_5() {
    let text = "x".repeat(4100 + 5);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_6() {
    let text = "x".repeat(4100 + 6);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_7() {
    let text = "x".repeat(4100 + 7);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_8() {
    let text = "x".repeat(4100 + 8);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_9() {
    let text = "x".repeat(4100 + 9);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_10() {
    let text = "x".repeat(4100 + 10);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_11() {
    let text = "x".repeat(4100 + 11);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_12() {
    let text = "x".repeat(4100 + 12);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_13() {
    let text = "x".repeat(4100 + 13);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_14() {
    let text = "x".repeat(4100 + 14);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_15() {
    let text = "x".repeat(4100 + 15);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_16() {
    let text = "x".repeat(4100 + 16);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_17() {
    let text = "x".repeat(4100 + 17);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_18() {
    let text = "x".repeat(4100 + 18);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_19() {
    let text = "x".repeat(4100 + 19);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_20() {
    let text = "x".repeat(4100 + 20);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_21() {
    let text = "x".repeat(4100 + 21);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_22() {
    let text = "x".repeat(4100 + 22);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_23() {
    let text = "x".repeat(4100 + 23);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_24() {
    let text = "x".repeat(4100 + 24);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_25() {
    let text = "x".repeat(4100 + 25);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_26() {
    let text = "x".repeat(4100 + 26);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_27() {
    let text = "x".repeat(4100 + 27);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_28() {
    let text = "x".repeat(4100 + 28);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_29() {
    let text = "x".repeat(4100 + 29);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_30() {
    let text = "x".repeat(4100 + 30);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_31() {
    let text = "x".repeat(4100 + 31);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_32() {
    let text = "x".repeat(4100 + 32);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_33() {
    let text = "x".repeat(4100 + 33);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_34() {
    let text = "x".repeat(4100 + 34);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_35() {
    let text = "x".repeat(4100 + 35);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_36() {
    let text = "x".repeat(4100 + 36);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_37() {
    let text = "x".repeat(4100 + 37);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_38() {
    let text = "x".repeat(4100 + 38);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_39() {
    let text = "x".repeat(4100 + 39);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_40() {
    let text = "x".repeat(4100 + 40);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_41() {
    let text = "x".repeat(4100 + 41);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_42() {
    let text = "x".repeat(4100 + 42);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_43() {
    let text = "x".repeat(4100 + 43);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_44() {
    let text = "x".repeat(4100 + 44);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_45() {
    let text = "x".repeat(4100 + 45);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_46() {
    let text = "x".repeat(4100 + 46);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_47() {
    let text = "x".repeat(4100 + 47);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_48() {
    let text = "x".repeat(4100 + 48);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_49() {
    let text = "x".repeat(4100 + 49);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_50() {
    let text = "x".repeat(4100 + 50);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_51() {
    let text = "x".repeat(4100 + 51);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_52() {
    let text = "x".repeat(4100 + 52);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_53() {
    let text = "x".repeat(4100 + 53);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_54() {
    let text = "x".repeat(4100 + 54);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_55() {
    let text = "x".repeat(4100 + 55);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_56() {
    let text = "x".repeat(4100 + 56);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_57() {
    let text = "x".repeat(4100 + 57);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_58() {
    let text = "x".repeat(4100 + 58);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_59() {
    let text = "x".repeat(4100 + 59);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_60() {
    let text = "x".repeat(4100 + 60);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_61() {
    let text = "x".repeat(4100 + 61);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_62() {
    let text = "x".repeat(4100 + 62);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_63() {
    let text = "x".repeat(4100 + 63);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_64() {
    let text = "x".repeat(4100 + 64);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_65() {
    let text = "x".repeat(4100 + 65);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_66() {
    let text = "x".repeat(4100 + 66);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_67() {
    let text = "x".repeat(4100 + 67);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_68() {
    let text = "x".repeat(4100 + 68);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_69() {
    let text = "x".repeat(4100 + 69);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_70() {
    let text = "x".repeat(4100 + 70);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_71() {
    let text = "x".repeat(4100 + 71);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_72() {
    let text = "x".repeat(4100 + 72);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_73() {
    let text = "x".repeat(4100 + 73);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_74() {
    let text = "x".repeat(4100 + 74);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_75() {
    let text = "x".repeat(4100 + 75);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_76() {
    let text = "x".repeat(4100 + 76);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_77() {
    let text = "x".repeat(4100 + 77);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_78() {
    let text = "x".repeat(4100 + 78);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_79() {
    let text = "x".repeat(4100 + 79);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_80() {
    let text = "x".repeat(4100 + 80);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_81() {
    let text = "x".repeat(4100 + 81);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_82() {
    let text = "x".repeat(4100 + 82);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_83() {
    let text = "x".repeat(4100 + 83);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_84() {
    let text = "x".repeat(4100 + 84);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_85() {
    let text = "x".repeat(4100 + 85);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_86() {
    let text = "x".repeat(4100 + 86);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_87() {
    let text = "x".repeat(4100 + 87);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_88() {
    let text = "x".repeat(4100 + 88);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_89() {
    let text = "x".repeat(4100 + 89);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_90() {
    let text = "x".repeat(4100 + 90);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_91() {
    let text = "x".repeat(4100 + 91);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_92() {
    let text = "x".repeat(4100 + 92);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_93() {
    let text = "x".repeat(4100 + 93);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_94() {
    let text = "x".repeat(4100 + 94);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_95() {
    let text = "x".repeat(4100 + 95);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_96() {
    let text = "x".repeat(4100 + 96);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_97() {
    let text = "x".repeat(4100 + 97);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_98() {
    let text = "x".repeat(4100 + 98);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_99() {
    let text = "x".repeat(4100 + 99);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_100() {
    let text = "x".repeat(4100 + 100);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_101() {
    let text = "x".repeat(4100 + 101);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_102() {
    let text = "x".repeat(4100 + 102);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_103() {
    let text = "x".repeat(4100 + 103);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_104() {
    let text = "x".repeat(4100 + 104);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_105() {
    let text = "x".repeat(4100 + 105);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_106() {
    let text = "x".repeat(4100 + 106);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_107() {
    let text = "x".repeat(4100 + 107);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_108() {
    let text = "x".repeat(4100 + 108);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_109() {
    let text = "x".repeat(4100 + 109);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_110() {
    let text = "x".repeat(4100 + 110);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_111() {
    let text = "x".repeat(4100 + 111);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_112() {
    let text = "x".repeat(4100 + 112);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_113() {
    let text = "x".repeat(4100 + 113);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_114() {
    let text = "x".repeat(4100 + 114);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_115() {
    let text = "x".repeat(4100 + 115);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_116() {
    let text = "x".repeat(4100 + 116);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_117() {
    let text = "x".repeat(4100 + 117);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_118() {
    let text = "x".repeat(4100 + 118);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_119() {
    let text = "x".repeat(4100 + 119);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_120() {
    let text = "x".repeat(4100 + 120);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_121() {
    let text = "x".repeat(4100 + 121);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_122() {
    let text = "x".repeat(4100 + 122);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_123() {
    let text = "x".repeat(4100 + 123);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_124() {
    let text = "x".repeat(4100 + 124);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_125() {
    let text = "x".repeat(4100 + 125);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_126() {
    let text = "x".repeat(4100 + 126);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_127() {
    let text = "x".repeat(4100 + 127);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_128() {
    let text = "x".repeat(4100 + 128);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_129() {
    let text = "x".repeat(4100 + 129);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_130() {
    let text = "x".repeat(4100 + 130);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_131() {
    let text = "x".repeat(4100 + 131);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_132() {
    let text = "x".repeat(4100 + 132);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_133() {
    let text = "x".repeat(4100 + 133);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_134() {
    let text = "x".repeat(4100 + 134);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_135() {
    let text = "x".repeat(4100 + 135);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_136() {
    let text = "x".repeat(4100 + 136);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_137() {
    let text = "x".repeat(4100 + 137);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_138() {
    let text = "x".repeat(4100 + 138);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_139() {
    let text = "x".repeat(4100 + 139);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_140() {
    let text = "x".repeat(4100 + 140);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_141() {
    let text = "x".repeat(4100 + 141);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_142() {
    let text = "x".repeat(4100 + 142);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_143() {
    let text = "x".repeat(4100 + 143);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_144() {
    let text = "x".repeat(4100 + 144);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_145() {
    let text = "x".repeat(4100 + 145);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_146() {
    let text = "x".repeat(4100 + 146);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_147() {
    let text = "x".repeat(4100 + 147);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_148() {
    let text = "x".repeat(4100 + 148);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_149() {
    let text = "x".repeat(4100 + 149);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_150() {
    let text = "x".repeat(4100 + 150);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_151() {
    let text = "x".repeat(4100 + 151);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_152() {
    let text = "x".repeat(4100 + 152);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_153() {
    let text = "x".repeat(4100 + 153);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_154() {
    let text = "x".repeat(4100 + 154);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_155() {
    let text = "x".repeat(4100 + 155);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_156() {
    let text = "x".repeat(4100 + 156);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_157() {
    let text = "x".repeat(4100 + 157);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_158() {
    let text = "x".repeat(4100 + 158);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_159() {
    let text = "x".repeat(4100 + 159);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_160() {
    let text = "x".repeat(4100 + 160);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_161() {
    let text = "x".repeat(4100 + 161);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_162() {
    let text = "x".repeat(4100 + 162);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_163() {
    let text = "x".repeat(4100 + 163);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_164() {
    let text = "x".repeat(4100 + 164);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_165() {
    let text = "x".repeat(4100 + 165);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_166() {
    let text = "x".repeat(4100 + 166);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_167() {
    let text = "x".repeat(4100 + 167);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_168() {
    let text = "x".repeat(4100 + 168);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_169() {
    let text = "x".repeat(4100 + 169);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_170() {
    let text = "x".repeat(4100 + 170);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_171() {
    let text = "x".repeat(4100 + 171);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_172() {
    let text = "x".repeat(4100 + 172);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_173() {
    let text = "x".repeat(4100 + 173);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_174() {
    let text = "x".repeat(4100 + 174);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_175() {
    let text = "x".repeat(4100 + 175);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_176() {
    let text = "x".repeat(4100 + 176);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_177() {
    let text = "x".repeat(4100 + 177);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_178() {
    let text = "x".repeat(4100 + 178);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_179() {
    let text = "x".repeat(4100 + 179);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_180() {
    let text = "x".repeat(4100 + 180);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_181() {
    let text = "x".repeat(4100 + 181);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_182() {
    let text = "x".repeat(4100 + 182);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_183() {
    let text = "x".repeat(4100 + 183);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_184() {
    let text = "x".repeat(4100 + 184);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_185() {
    let text = "x".repeat(4100 + 185);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_186() {
    let text = "x".repeat(4100 + 186);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_187() {
    let text = "x".repeat(4100 + 187);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_188() {
    let text = "x".repeat(4100 + 188);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_189() {
    let text = "x".repeat(4100 + 189);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_190() {
    let text = "x".repeat(4100 + 190);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_191() {
    let text = "x".repeat(4100 + 191);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_192() {
    let text = "x".repeat(4100 + 192);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_193() {
    let text = "x".repeat(4100 + 193);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_194() {
    let text = "x".repeat(4100 + 194);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_195() {
    let text = "x".repeat(4100 + 195);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_196() {
    let text = "x".repeat(4100 + 196);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_197() {
    let text = "x".repeat(4100 + 197);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_198() {
    let text = "x".repeat(4100 + 198);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_199() {
    let text = "x".repeat(4100 + 199);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_200() {
    let text = "x".repeat(4100 + 200);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_201() {
    let text = "x".repeat(4100 + 201);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_202() {
    let text = "x".repeat(4100 + 202);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_203() {
    let text = "x".repeat(4100 + 203);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_204() {
    let text = "x".repeat(4100 + 204);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_205() {
    let text = "x".repeat(4100 + 205);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_206() {
    let text = "x".repeat(4100 + 206);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_207() {
    let text = "x".repeat(4100 + 207);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_208() {
    let text = "x".repeat(4100 + 208);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_209() {
    let text = "x".repeat(4100 + 209);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_210() {
    let text = "x".repeat(4100 + 210);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_211() {
    let text = "x".repeat(4100 + 211);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_212() {
    let text = "x".repeat(4100 + 212);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_213() {
    let text = "x".repeat(4100 + 213);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_214() {
    let text = "x".repeat(4100 + 214);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_215() {
    let text = "x".repeat(4100 + 215);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_216() {
    let text = "x".repeat(4100 + 216);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_217() {
    let text = "x".repeat(4100 + 217);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_218() {
    let text = "x".repeat(4100 + 218);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_219() {
    let text = "x".repeat(4100 + 219);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_220() {
    let text = "x".repeat(4100 + 220);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_221() {
    let text = "x".repeat(4100 + 221);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_222() {
    let text = "x".repeat(4100 + 222);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_223() {
    let text = "x".repeat(4100 + 223);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_224() {
    let text = "x".repeat(4100 + 224);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_225() {
    let text = "x".repeat(4100 + 225);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_226() {
    let text = "x".repeat(4100 + 226);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_227() {
    let text = "x".repeat(4100 + 227);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_228() {
    let text = "x".repeat(4100 + 228);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_229() {
    let text = "x".repeat(4100 + 229);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_230() {
    let text = "x".repeat(4100 + 230);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_231() {
    let text = "x".repeat(4100 + 231);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_232() {
    let text = "x".repeat(4100 + 232);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_233() {
    let text = "x".repeat(4100 + 233);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_234() {
    let text = "x".repeat(4100 + 234);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_235() {
    let text = "x".repeat(4100 + 235);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_236() {
    let text = "x".repeat(4100 + 236);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_237() {
    let text = "x".repeat(4100 + 237);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_238() {
    let text = "x".repeat(4100 + 238);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_239() {
    let text = "x".repeat(4100 + 239);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_240() {
    let text = "x".repeat(4100 + 240);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_241() {
    let text = "x".repeat(4100 + 241);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_242() {
    let text = "x".repeat(4100 + 242);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_243() {
    let text = "x".repeat(4100 + 243);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_244() {
    let text = "x".repeat(4100 + 244);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_245() {
    let text = "x".repeat(4100 + 245);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_246() {
    let text = "x".repeat(4100 + 246);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_247() {
    let text = "x".repeat(4100 + 247);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_248() {
    let text = "x".repeat(4100 + 248);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_249() {
    let text = "x".repeat(4100 + 249);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}

#[cfg(feature = "telegram")]
#[test]
fn telegram_split_case_250() {
    let text = "x".repeat(4100 + 250);
    let chunks = TelegramChannel::split_message(&text);
    assert!(chunks.len() >= 2);
    assert!(chunks.iter().all(|chunk| chunk.len() <= 4096));
}
