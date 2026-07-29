//! Translation plugin.
//!
//! Provides translation candidates (Chinese → English / English → Chinese).
//! Uses a small built-in dictionary. Extend with cloud API for production.

use crate::Plugin;
use skyme_common::Candidate;

/// Small bilingual dictionary (Chinese → English).
const DICT_ZH_EN: &[(&str, &str)] = &[
    ("你好", "hello"), ("谢谢", "thank you"), ("再见", "goodbye"),
    ("早上好", "good morning"), ("晚上好", "good evening"),
    ("对不起", "sorry"), ("没关系", "it's ok"), ("请", "please"),
    ("是", "yes"), ("不是", "no"), ("好", "good"), ("坏", "bad"),
    ("大", "big"), ("小", "small"), ("多", "many"), ("少", "few"),
    ("爱", "love"), ("恨", "hate"), ("想", "think"), ("说", "speak"),
    ("吃", "eat"), ("喝", "drink"), ("走", "walk"), ("跑", "run"),
    ("看", "see"), ("听", "hear"), ("读", "read"), ("写", "write"),
    ("工作", "work"), ("学习", "study"), ("睡觉", "sleep"),
    ("今天", "today"), ("明天", "tomorrow"), ("昨天", "yesterday"),
    ("星期", "week"), ("月", "month"), ("年", "year"),
    ("一", "one"), ("二", "two"), ("三", "three"), ("四", "four"),
    ("五", "five"), ("六", "six"), ("七", "seven"), ("八", "eight"),
    ("九", "nine"), ("十", "ten"),
];

/// English → Chinese (reverse index)
const DICT_EN_ZH: &[(&str, &str)] = &[
    ("hello", "你好"), ("hi", "你好"), ("thank", "谢谢"), ("bye", "再见"),
    ("good", "好"), ("bad", "坏"), ("big", "大"), ("small", "小"),
    ("love", "爱"), ("hate", "恨"), ("yes", "是"), ("no", "不"),
    ("please", "请"), ("sorry", "对不起"), ("ok", "好"),
    ("today", "今天"), ("tomorrow", "明天"), ("now", "现在"),
    ("water", "水"), ("food", "食物"), ("tea", "茶"), ("coffee", "咖啡"),
    ("book", "书"), ("pen", "笔"), ("computer", "电脑"), ("phone", "电话"),
    ("one", "一"), ("two", "二"), ("three", "三"), ("four", "四"),
    ("five", "五"), ("six", "六"), ("seven", "七"), ("eight", "八"),
    ("nine", "九"), ("ten", "十"),
];

pub struct TranslatorPlugin;

impl Plugin for TranslatorPlugin {
    fn name(&self) -> &str { "translator" }

    fn on_candidate(&mut self, candidates: &mut Vec<Candidate>) {
        let mut translations: Vec<Candidate> = Vec::new();

        for cand in candidates.iter() {
            let text = cand.text.trim();
            if text.is_empty() { continue; }

            // Try Chinese → English
            for &(zh, en) in DICT_ZH_EN {
                if text.contains(zh) {
                    translations.push(Candidate {
                        text: en.to_owned(),
                        comment: format!("🌐 translate: {}", zh),
                        index: 0, quality: cand.quality + 0.05,
                    });
                    break;
                }
            }

            // Try English → Chinese
            let lower = text.to_lowercase();
            for &(en, zh) in DICT_EN_ZH {
                if lower.contains(en) {
                    translations.push(Candidate {
                        text: zh.to_owned(),
                        comment: format!("🌐 translate: {}", en),
                        index: 0, quality: cand.quality + 0.05,
                    });
                    break;
                }
            }
        }

        if !translations.is_empty() {
            let mut result = translations;
            result.extend(candidates.drain(..));
            *candidates = result;
        }
    }
}
