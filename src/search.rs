use crate::data::{Command, Category, Platform};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 搜索结果
pub struct SearchResult<'a> {
    pub platform: &'a Platform,
    pub category: &'a Category,
    pub command: &'a Command,
    pub score: i64,
}

/// 搜索引擎
pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default().ignore_case(),
        }
    }

    /// 搜索命令，返回匹配结果（按分数排序）
    pub fn search<'a>(
        &self,
        query: &str,
        platforms: &'a [Platform],
    ) -> Vec<SearchResult<'a>> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for platform in platforms {
            for category in &platform.categories {
                for command in &category.commands {
                    // 对命令名称、摘要、示例代码、tips 进行模糊匹配
                    let name_score = self.matcher.fuzzy_match(&command.name, query);
                    let summary_score = self.matcher.fuzzy_match(&command.summary, query);

                    // 搜索 example code
                    let example_score = command
                        .examples
                        .iter()
                        .filter_map(|e| self.matcher.fuzzy_match(&e.code, query))
                        .max();

                    // 搜索 tips
                    let tips_score = command
                        .tips
                        .iter()
                        .filter_map(|t| self.matcher.fuzzy_match(t, query))
                        .max();

                    // 搜索 tags（场景标签）
                    let tags_score = command
                        .tags
                        .iter()
                        .filter_map(|t| self.matcher.fuzzy_match(t, query))
                        .max();

                    // 综合评分：名称 x3, 摘要 x1, examples x1, tips x0.5, tags x2
                    let mut score: i64 = 0;
                    let mut matched = false;

                    if let Some(n) = name_score {
                        score += n * 3;
                        matched = true;
                    }
                    if let Some(s) = summary_score {
                        score += s;
                        matched = true;
                    }
                    if let Some(e) = example_score {
                        score += e;
                        matched = true;
                    }
                    if let Some(t) = tips_score {
                        score += t / 2;
                        matched = true;
                    }
                    if let Some(tg) = tags_score {
                        score += tg * 2;
                        matched = true;
                    }

                    if !matched {
                        continue;
                    }

                    results.push(SearchResult {
                        platform,
                        category,
                        command,
                        score,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}
