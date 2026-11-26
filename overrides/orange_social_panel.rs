use freya::prelude::*;

use super::orange_social::OrangeSocialPanel;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelMode {
    Launcher,
    Window,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelView {
    List,
    Chat,
    Blocked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelTab {
    Chats,
    Friends,
    Groups,
    Special,
}

#[derive(PartialEq)]
pub(super) struct SocialPanel {
    pub mode: PanelMode,
    pub expanded: bool,
}

impl Component for SocialPanel {
    fn render(&self) -> impl IntoElement {
        OrangeSocialPanel.into_element()
    }
}
