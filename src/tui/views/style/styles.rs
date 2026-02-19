use anyhow::{Error, Result};

use crate::{
    configs::{self},
    tui::views::style::{
        ColorStyle, ListStyle, ModalStyle, PreviewStyle, ScreenScaffoldStyle, SearchBarStyle,
        StatusStyle,
    },
};

#[allow(dead_code)]
pub struct Styles {
    pub list: ListStyle,
    pub colors: ColorStyle,
    pub preview: PreviewStyle,
    pub modal: ModalStyle,
    pub status: StatusStyle,
    pub search_bar_style: SearchBarStyle,
    pub screen_scaffold_style: ScreenScaffoldStyle,
}

impl TryFrom<&configs::Styles> for Styles {
    type Error = Error;

    fn try_from(styles: &configs::Styles) -> Result<Styles> {
        let styles = Self {
            list: ListStyle::from(&styles.list),
            colors: ColorStyle::try_from(&styles.colors)?,
            preview: PreviewStyle::from(&styles.preview),
            modal: ModalStyle::from(&styles.modal),
            status: StatusStyle::from(&styles.status),
            search_bar_style: SearchBarStyle::from(&styles.search_bar),
            screen_scaffold_style: ScreenScaffoldStyle::from(&styles.screen_scaffold),
        };
        Ok(styles)
    }
}
