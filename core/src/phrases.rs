//! Sentences the user must retype to quit early, in each supported language.

use crate::persona::norm_lang;

pub const SENTENCES_VI: [&str; 8] = [
    "Mình đã hứa với bản thân sẽ làm xong việc này, và lời hứa đó quan trọng hơn cơn buồn chán nhất thời.",
    "Mỗi lần bỏ dở giữa chừng là một lần tập thói quen bỏ cuộc, nên mình chọn ở lại thêm một chút nữa.",
    "Sự tập trung hôm nay là món quà mình dành cho chính mình của tương lai, đừng bán nó rẻ cho vài phút lướt mạng.",
    "Cảm giác muốn thoát ra sẽ qua đi sau vài phút, còn thành quả của việc làm trọn thời gian thì ở lại rất lâu.",
    "Mình không cần cảm hứng, mình chỉ cần ngồi xuống và làm tiếp đến khi đồng hồ kết thúc.",
    "Những điều khiến mình muốn bỏ đi lúc này gần như chẳng khẩn cấp gì, chúng hoàn toàn có thể chờ đến giờ nghỉ.",
    "Người kiên trì không phải người không bao giờ chán, mà là người vẫn làm tiếp dù đang chán.",
    "Nếu việc thật sự khẩn cấp thì mình đã có lý do chính đáng, còn nếu không thì mình sẽ quay lại làm việc ngay.",
];

pub const SENTENCES_EN: [&str; 8] = [
    "I promised myself I would finish this, and that promise matters more than a passing wave of boredom.",
    "Every time I quit halfway I practise the habit of giving up, so I choose to stay a little longer.",
    "Focus today is a gift to my future self, so I will not sell it cheap for a few minutes of scrolling.",
    "The urge to leave will pass in a few minutes, while the result of finishing the full time lasts a long while.",
    "I do not need inspiration, I only need to sit down and keep going until the timer ends.",
    "Whatever is pulling me away right now is almost never urgent, and it can wait until my break.",
    "A persistent person is not someone who never gets bored, but someone who keeps working while bored.",
    "If something is truly urgent I already have a good reason, and if not I will go straight back to work.",
];

pub fn sentences(lang: &str) -> &'static [&'static str; 8] {
    if norm_lang(lang) == "en" {
        &SENTENCES_EN
    } else {
        &SENTENCES_VI
    }
}

/// Joins `n` different sentences (1 easy, 2 normal, 3 hard). `rand` returns a value in `[0, 1)`.
pub fn pick_phrase(mut rand: impl FnMut() -> f64, n: usize, lang: &str) -> String {
    let all = sentences(lang);
    let count = n.clamp(1, all.len());
    let mut used: Vec<usize> = Vec::new();
    while used.len() < count {
        let mut i = ((rand() * all.len() as f64).floor() as usize).min(all.len() - 1);
        while used.contains(&i) {
            i = (i + 1) % all.len();
        }
        used.push(i);
    }
    used.iter().map(|&i| all[i]).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrase_length_grows_with_level_and_never_repeats() {
        for lang in ["vi", "en"] {
            let [a, b, c] = [1, 2, 3].map(|n| pick_phrase(|| 0.0, n, lang));
            assert!(a.chars().count() < b.chars().count() && b.chars().count() < c.chars().count());
            assert!(a.chars().count() > 60);
            let parts: Vec<&str> = c.split(". ").collect();
            let unique: std::collections::HashSet<_> = parts.iter().collect();
            assert_eq!(unique.len(), parts.len());
        }
    }

    #[test]
    fn same_value_twice_still_gives_different_sentences() {
        let p = pick_phrase(|| 0.99, 3, "vi");
        assert_eq!(p.matches("Nếu việc thật sự").count(), 1);
    }

    #[test]
    fn language_picks_the_matching_sentence_set() {
        assert!(pick_phrase(|| 0.0, 1, "en").starts_with("I promised"));
        assert!(pick_phrase(|| 0.0, 1, "vi").starts_with("Mình đã hứa"));
        assert!(
            pick_phrase(|| 0.0, 1, "fr").starts_with("Mình đã hứa"),
            "unknown language falls back to Vietnamese"
        );
    }
}
