#![allow(dead_code)]
use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Theme {
    pub ui: UIColors,
    pub editor: EditorColors,
    pub overlay: OverlayColors,
    pub status_line: StatusLineColors,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UIColors {
    pub base_bg: String,
    pub base_text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditorColors {
    #[serde(rename = "bg")]
    pub bg: String,
    #[serde(rename = "current_line")]
    pub current_line: String,
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "comments")]
    pub comments: String,
    #[serde(rename = "keywords")]
    pub keywords: String,
    #[serde(rename = "string_literal")]
    pub string_literal: String,
    #[serde(rename = "number_literal")]
    pub number_literal: String,
    #[serde(rename = "line_numbers")]
    pub line_numbers: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverlayColors {
    #[serde(rename = "bg")]
    pub bg: String,
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusLineColors {
    #[serde(rename = "bg")]
    pub bg: String,
    #[serde(rename = "text")]
    pub text: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            ui: UIColors {
                base_bg: "#1a1d23".to_string(),  // Original dark background for UI
                base_text: "#e0e0e0".to_string(), // Soft light text color
            },
            editor: EditorColors {
                bg: "#1f2329".to_string(),          // Keeping the dark background for the editor
                current_line: "#333b47".to_string(), // Slightly lighter for the current line
                text: "#eaeaea".to_string(),         // Soft light gray text
                comments: "#6d7f6d".to_string(),     // Muted, darker pastel green for comments
                keywords: "#FFA07A".to_string(),     // Soft orangish pastel for keywords
                string_literal: "#98c379".to_string(), // Gentle pastel green for string literals
                number_literal: "#a6a9d6".to_string(), // Soft pastel purple for numbers
                line_numbers: "#767f8c".to_string(), // Darker, softer color for line numbers
            },
            overlay: OverlayColors {
                bg: "#282c34".to_string(),  // Dark background for overlays
                text: "#f8f8f2".to_string(), // Soft off-white text for overlays
            },
            status_line: StatusLineColors {
                bg: "#3b4048".to_string(),  // Subtle dark gray for status line background
                text: "#ffffff".to_string(), // White text for status line
            },
        }
    }
}

impl Theme {
    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim_start_matches('#');

        let r_str = &hex[0..2];
        let g_str = &hex[2..4];
        let b_str = &hex[4..6];

        let r = u8::from_str_radix(r_str, 16).expect("Invalid hex: {r_str}");
        let g = u8::from_str_radix(g_str, 16).expect("Invalid hex: {r_str}");
        let b = u8::from_str_radix(b_str, 16).expect("Invalid hex: {r_str}");

        (r, g, b)
    }

    pub fn hex_to_color(hex: &str) -> Color {
        let (r, g, b) = Self::hex_to_rgb(hex);
        
        Color::Rgb { r, g, b }
    }
}