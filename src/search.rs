use crate::data::{Category, Command, Platform};
use crate::prompts::PromptStore;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 命令搜索结果
pub struct CommandSearchResult<'a> {
    pub platform: &'a Platform,
    pub category: &'a Category,
    pub command: &'a Command,
    pub score: i64,
}

/// Prompt 搜索结果
pub struct PromptSearchResult<'a> {
    pub prompt: &'a crate::prompts::Prompt,
    pub score: i64,
}

/// Password 搜索结果
pub struct PasswordSearchResult<'a> {
    pub password: &'a crate::passwords::Password,
    pub score: i64,
}

/// 搜索引擎
pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

/// 对长文本做可搜索截断，避免超长 Prompt 内容拖慢模糊匹配。
fn searchable_text(text: &str, max_chars: usize, max_lines: usize) -> String {
    let mut lines = text.lines();
    let mut result = String::new();
    let mut line_count = 0;
    while line_count < max_lines {
        if let Some(line) = lines.next() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            line_count += 1;
        } else {
            break;
        }
    }
    if result.chars().count() > max_chars {
        result.chars().take(max_chars).collect()
    } else {
        result
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default().ignore_case(),
        }
    }

    /// 搜索命令，返回匹配结果（按分数排序）
    pub fn search_commands<'a>(
        &self,
        query: &str,
        platforms: &'a [Platform],
    ) -> Vec<CommandSearchResult<'a>> {
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

                    results.push(CommandSearchResult {
                        platform,
                        category,
                        command,
                        score,
                    });
                }
            }
        }

        results.sort_by_key(|a| std::cmp::Reverse(a.score));
        results
    }

    /// 搜索 Prompts，返回匹配结果（按分数排序）。
    /// 为避免超长内容拖慢每次按键，content 只取前 500 字符 / 前 20 行参与匹配。
    pub fn search_prompts<'a>(
        &self,
        query: &str,
        prompts: &'a PromptStore,
    ) -> Vec<PromptSearchResult<'a>> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        const CONTENT_MAX_CHARS: usize = 500;
        const CONTENT_MAX_LINES: usize = 20;

        for prompt in prompts.all() {
            let name_score = self.matcher.fuzzy_match(&prompt.name, query);
            let description_score = self.matcher.fuzzy_match(&prompt.description, query);
            let tags_score = prompt
                .tags
                .iter()
                .filter_map(|t| self.matcher.fuzzy_match(t, query))
                .max();

            let content_preview = searchable_text(&prompt.content, CONTENT_MAX_CHARS, CONTENT_MAX_LINES);
            let content_score = self.matcher.fuzzy_match(&content_preview, query);

            // 名称 x3, 描述 x1, tags x2, content x0.5
            let mut score: i64 = 0;
            let mut matched = false;

            if let Some(n) = name_score {
                score += n * 3;
                matched = true;
            }
            if let Some(d) = description_score {
                score += d;
                matched = true;
            }
            if let Some(tg) = tags_score {
                score += tg * 2;
                matched = true;
            }
            if let Some(c) = content_score {
                score += c / 2;
                matched = true;
            }

            if !matched {
                continue;
            }

            results.push(PromptSearchResult { prompt, score });
        }

        results.sort_by_key(|a| std::cmp::Reverse(a.score));
        results
    }

    /// 搜索 Passwords，返回匹配结果（按分数排序）。
    /// 仅搜索 name 和 description，不搜索加密的 value 字段。
    pub fn search_passwords<'a>(
        &self,
        query: &str,
        passwords: &'a crate::passwords::PasswordStore,
    ) -> Vec<PasswordSearchResult<'a>> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for password in passwords.all() {
            let name_score = self.matcher.fuzzy_match(&password.name, query);
            let description_score = self.matcher.fuzzy_match(&password.description, query);

            // 名称 x3, 描述 x1
            let mut score: i64 = 0;
            let mut matched = false;

            if let Some(n) = name_score {
                score += n * 3;
                matched = true;
            }
            if let Some(d) = description_score {
                score += d;
                matched = true;
            }

            if !matched {
                continue;
            }

            results.push(PasswordSearchResult { password, score });
        }

        results.sort_by_key(|a| std::cmp::Reverse(a.score));
        results
    }
}
