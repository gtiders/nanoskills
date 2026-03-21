use crate::domain::Skill;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{layout::Rect, widgets::TableState};

use super::view_model::{FilteredSkill, MatchHighlights, SkillViewModel, split_match_indices};

/// TUI application state for fuzzy filtering and preview rendering.
pub(super) struct App {
    pub(super) items: Vec<SkillViewModel>,
    pub(super) filtered_items: Vec<FilteredSkill>,
    pub(super) state: TableState,
    pub(super) search_input: String,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
    last_search: String,
    pub(super) table_area: Rect,
}

impl App {
    /// Build a new application state from indexed skills.
    pub(super) fn new(items: Vec<Skill>) -> Self {
        let items: Vec<_> = items.into_iter().map(SkillViewModel::new).collect();
        let mut state = TableState::default();
        state.select((!items.is_empty()).then_some(0));

        let filtered_items = (0..items.len())
            .map(|item_index| FilteredSkill {
                item_index,
                score: 0,
                highlights: MatchHighlights::default(),
            })
            .collect();

        Self {
            items,
            filtered_items,
            state,
            search_input: String::new(),
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
            last_search: String::new(),
            table_area: Rect::default(),
        }
    }

    /// Recompute fuzzy matches when the query changes.
    pub(super) fn filter_items(&mut self) {
        if self.search_input == self.last_search {
            return;
        }

        if self.search_input.is_empty() {
            self.filtered_items = (0..self.items.len())
                .map(|item_index| FilteredSkill {
                    item_index,
                    score: 0,
                    highlights: MatchHighlights::default(),
                })
                .collect();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(item_index, item)| {
                    self.matcher
                        .fuzzy_indices(&item.projection.haystack, &self.search_input)
                        .map(|(score, indices)| FilteredSkill {
                            item_index,
                            score,
                            highlights: split_match_indices(&indices, &item.projection),
                        })
                })
                .collect();

            self.filtered_items.sort_by(|left, right| {
                right
                    .score
                    .cmp(&left.score)
                    .then_with(|| left.item_index.cmp(&right.item_index))
            });
        }

        self.state
            .select((!self.filtered_items.is_empty()).then_some(0));
        self.last_search.clone_from(&self.search_input);
    }

    pub(super) fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let next = match self.state.selected() {
            Some(index) if index + 1 < self.filtered_items.len() => index + 1,
            Some(_) | None => 0,
        };

        self.state.select(Some(next));
    }

    pub(super) fn previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let previous = match self.state.selected() {
            Some(0) | None => self.filtered_items.len() - 1,
            Some(index) => index - 1,
        };

        self.state.select(Some(previous));
    }

    pub(super) fn select_index(&mut self, index: usize) {
        if index < self.filtered_items.len() {
            self.state.select(Some(index));
        }
    }

    pub(super) fn selected_skill(&self) -> Option<&Skill> {
        self.state
            .selected()
            .and_then(|selected| self.filtered_items.get(selected))
            .and_then(|item| self.items.get(item.item_index))
            .map(|item| &item.skill)
    }

    pub(super) fn selected_view(&self) -> Option<&SkillViewModel> {
        self.state
            .selected()
            .and_then(|selected| self.filtered_items.get(selected))
            .and_then(|item| self.items.get(item.item_index))
    }
}
