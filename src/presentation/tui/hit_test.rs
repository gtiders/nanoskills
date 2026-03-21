use ratatui::{layout::Rect, widgets::TableState};

pub(super) fn calculate_clicked_index(
    mouse_y: u16,
    table_rect: Rect,
    table_state: &TableState,
    item_count: usize,
) -> Option<usize> {
    let table_top = table_rect.top();
    let table_bottom = table_rect.bottom();

    if mouse_y < table_top || mouse_y >= table_bottom {
        return None;
    }

    let header_height = 1u16;
    let border_height = 1u16;

    // 鼠标命中的是终端绝对坐标，而表格选择需要“可见数据行”的相对索引。
    // 这里显式扣掉边框和表头高度，才能和 ratatui 内部滚动 offset 对齐。
    if mouse_y < table_top + border_height + header_height {
        return None;
    }

    let clicked_row = (mouse_y - table_top - border_height - header_height) as usize;
    let absolute_index = table_state.offset() + clicked_row;

    (absolute_index < item_count).then_some(absolute_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_state_with_offset(offset: usize) -> TableState {
        TableState::default().with_offset(offset)
    }

    #[test]
    fn test_click_above_header_returns_none() {
        let state = table_state_with_offset(0);
        let rect = Rect::new(0, 2, 40, 10);

        // 点击边框和表头区域时，不能错误选中第一行数据。
        assert_eq!(calculate_clicked_index(2, rect, &state, 5), None);
        assert_eq!(calculate_clicked_index(3, rect, &state, 5), None);
    }

    #[test]
    fn test_click_on_visible_row_respects_scroll_offset() {
        let state = table_state_with_offset(4);
        let rect = Rect::new(0, 1, 40, 10);

        // 第一个可见数据行在 offset=4 时应映射到绝对索引 4。
        assert_eq!(calculate_clicked_index(3, rect, &state, 20), Some(4));
        // 第二个可见数据行应继续顺延到绝对索引 5。
        assert_eq!(calculate_clicked_index(4, rect, &state, 20), Some(5));
    }

    #[test]
    fn test_click_beyond_item_count_returns_none() {
        let state = table_state_with_offset(3);
        let rect = Rect::new(0, 0, 40, 10);

        // 可见行坐标即使合法，只要已经超过真实数据长度，也必须返回 None 防越界。
        assert_eq!(calculate_clicked_index(4, rect, &state, 4), None);
    }
}
