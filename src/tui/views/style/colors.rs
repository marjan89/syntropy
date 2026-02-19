use anyhow::{Context, Error, Result, ensure};
use ratatui::style::Color;

use crate::configs::style::{Colors, DEFAULT_COLOR};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ColorStyle {
    pub highlights_text: Color,
    pub highlights_background: Color,
    pub borders: Color,
    pub borders_list: Color,
    pub borders_preview: Color,
    pub borders_search: Color,
    pub borders_status: Color,
    pub borders_modal: Color,
    pub text: Color,
    pub text_list: Color,
    pub text_preview: Color,
    pub text_search: Color,
    pub text_status: Color,
    pub text_modal: Color,
    pub background: Color,
    pub background_list: Color,
    pub background_preview: Color,
    pub background_search: Color,
    pub background_status: Color,
    pub background_modal: Color,
}

impl TryFrom<&Colors> for ColorStyle {
    type Error = Error;

    fn try_from(colors: &Colors) -> Result<Self> {
        let borders = parse_color(&colors.borders)?.unwrap_or(Color::Reset);
        let text = parse_color(&colors.text)?.unwrap_or(Color::Reset);
        let background = parse_color(&colors.background)?.unwrap_or(Color::Reset);

        let color_style = Self {
            highlights_text: parse_color(&colors.highlights_text)?.unwrap_or(Color::Reset),
            highlights_background: parse_color(&colors.highlights_background)?
                .unwrap_or(Color::Reset),
            borders,
            borders_list: parse_color(&colors.borders_list)?.unwrap_or(borders),
            borders_preview: parse_color(&colors.borders_preview)?.unwrap_or(borders),
            borders_search: parse_color(&colors.borders_search)?.unwrap_or(borders),
            borders_status: parse_color(&colors.borders_status)?.unwrap_or(borders),
            borders_modal: parse_color(&colors.borders_modal)?.unwrap_or(borders),
            text,
            text_list: parse_color(&colors.text_list)?.unwrap_or(text),
            text_preview: parse_color(&colors.text_preview)?.unwrap_or(text),
            text_search: parse_color(&colors.text_search)?.unwrap_or(text),
            text_status: parse_color(&colors.text_status)?.unwrap_or(text),
            text_modal: parse_color(&colors.text_modal)?.unwrap_or(text),
            background,
            background_list: parse_color(&colors.background_list)?.unwrap_or(background),
            background_preview: parse_color(&colors.background_preview)?.unwrap_or(background),
            background_search: parse_color(&colors.background_search)?.unwrap_or(background),
            background_status: parse_color(&colors.background_status)?.unwrap_or(background),
            background_modal: parse_color(&colors.background_modal)?.unwrap_or(background),
        };

        Ok(color_style)
    }
}

pub fn parse_color(color: &str) -> Result<Option<Color>> {
    let normalized_color = color.trim();

    if normalized_color.is_empty() {
        return Ok(None);
    }

    let normalized_color = normalized_color.to_lowercase();
    let color = match normalized_color.as_str() {
        DEFAULT_COLOR => Some(Color::Reset),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => Some(parse_hex(&normalized_color)?),
    };

    Ok(color)
}

fn parse_hex(color: &str) -> Result<Color> {
    ensure!(
        color.len() == 7,
        format!(
            "Color {} format is not correct, it must be in format of #ffffff",
            color
        )
    );
    ensure!(
        color.starts_with('#'),
        format!(
            "Color {} format is not correct, it must be in format of #ffffff",
            color
        )
    );

    let hex = &color[1..];

    u32::from_str_radix(hex, 16).with_context(|| {
        format!(
            "Color {} is not a valid hex string, it must be in format of #ffffff",
            color
        )
    })?;

    let red = &hex[0..2];
    let green = &hex[2..4];
    let blue = &hex[4..6];

    let r = u8::from_str_radix(red, 16)
        .with_context(|| format!("Parsing red channel for {} failed", red))?;
    let g = u8::from_str_radix(green, 16)
        .with_context(|| format!("Parsing green channel for {} failed", green))?;
    let b = u8::from_str_radix(blue, 16)
        .with_context(|| format!("Parsing blue channel for {} failed", blue))?;

    Ok(Color::Rgb(r, g, b))
}
