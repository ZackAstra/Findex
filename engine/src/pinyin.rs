/// Pinyin matching module for Chinese characters.
use std::collections::HashMap;

fn build_pinyin_map() -> HashMap<char, String> {
    let mut map = HashMap::new();
    let pairs = vec![
        ('报', "b"), ('告', "g"), ('文', "w"), ('件', "j"),
        ('文', "w"), ('档', "d"), ('目', "m"), ('录', "l"),
        ('文', "w"), ('件', "j"), ('夹', "j"), ('项', "x"),
        ('目', "m"), ('测', "c"), ('试', "s"), ('源', "y"),
        ('代', "d"), ('码', "m"), ('工', "g"), ('程', "c"),
        ('项', "x"), ('目', "m"), ('数', "s"), ('据', "j"),
        ('库', "k"), ('数', "s"), ('据', "j"), ('文', "w"),
        ('件', "j"), ('图', "t"), ('片', "p"), ('视', "s"),
        ('频', "p"), ('音', "y"), ('乐', "l"), ('视', "s"),
        ('频', "p"), ('文', "w"), ('本', "b"), ('表', "b"),
        ('格', "g"), ('演', "y"), ('示', "s"), ('幻', "h"),
        ('灯', "d"), ('片', "p"), ('简', "j"), ('报', "b"),
        ('合', "h"), ('同', "t"), ('发', "f"), ('票', "p"),
        ('合', "h"), ('约', "y"), ('简', "j"), ('历', "l"),
        ('备', "b"), ('份', "f"), ('版', "b"), ('本', "b"),
        ('配', "p"), ('置', "z"), ('设', "s"), ('置', "z"),
        ('日', "r"), ('志', "z"), ('日', "r"), ('期', "q"),
        ('时', "s"), ('间', "j"), ('大', "d"), ('小', "x"),
        ('名', "m"), ('称', "c"), ('类', "l"), ('型', "x"),
        ('路', "l"), ('径', "j"), ('扩', "k"), ('展', "z"),
        ('后', "h"), ('缀', "z"), ('属', "s"), ('性', "x"),
        ('创', "c"), ('建', "j"), ('修', "x"), ('改', "g"),
        ('访', "f"), ('问', "w"), ('隐', "y"), ('藏', "c"),
        ('只', "z"), ('读', "d"), ('系', "x"), ('统', "t"),
        ('目', "m"), ('录', "l"), ('卷', "j"), ('搜', "s"),
        ('索', "s"), ('结', "j"), ('果', "g"), ('过', "g"),
        ('滤', "l"), ('排', "p"), ('序', "x"), ('复', "f"),
        ('制', "z"), ('粘', "z"), ('贴', "t"), ('删', "s"),
        ('除', "c"), ('重', "c"), ('命', "m"), ('名', "m"),
        ('属', "s"), ('性', "x"), ('打', "d"), ('开', "k"),
        ('保', "b"), ('存', "c"), ('关', "g"), ('闭', "b"),
        ('退', "t"), ('出', "c"), ('刷', "s"), ('新', "x"),
        ('加', "j"), ('载', "z"), ('同', "t"), ('步', "b"),
    ];
    for (ch, py) in pairs {
        map.insert(ch, py.to_string());
    }
    map
}

pub fn get_pinyin_initials(text: &str) -> String {
    let map = build_pinyin_map();
    text.chars().filter_map(|c| map.get(&c)).cloned().collect()
}

pub fn has_chinese(text: &str) -> bool {
    text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}')
}

pub fn match_pinyin(file_name: &str, query: &str) -> bool {
    let initials = get_pinyin_initials(file_name);
    initials.contains(&query.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinyin_initials() {
        assert_eq!(get_pinyin_initials("报告"), "bg");
        assert_eq!(get_pinyin_initials("测试"), "cs");
        assert_eq!(get_pinyin_initials("文档"), "wd");
    }

    #[test]
    fn test_match_pinyin() {
        assert!(match_pinyin("报告.docx", "bg"));
        assert!(match_pinyin("测试文档.txt", "cswd"));
        assert!(!match_pinyin("报告.docx", "cs"));
    }

    #[test]
    fn test_has_chinese() {
        assert!(has_chinese("报告"));
        assert!(!has_chinese("report"));
    }
}
