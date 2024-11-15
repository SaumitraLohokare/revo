#![allow(dead_code)]
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

pub fn default_theme() -> Theme {
    Theme {
        ui: UIColors {
            base_bg: "#1a1d23".to_string(),  // Darker background for the UI
            base_text: "#e0e0e0".to_string(), // Light gray for text
        },
        editor: EditorColors {
            bg: "#1f2329".to_string(),          // Darker background for editor
            current_line: "#333b47".to_string(), // Slightly lighter for the current line
            text: "#eaeaea".to_string(),         // Light text color
            comments: "#4b8b3b".to_string(),     // Darker green for comments
            keywords: "#FFA500".to_string(),     // Yellowish orange for keywords
            string_literal: "#6a9955".to_string(), // Green for strings
            number_literal: "#bd93f9".to_string(), // Purple for numbers
        },
        overlay: OverlayColors {
            bg: "#282c34".to_string(),  // Dark background for overlays
            text: "#f8f8f2".to_string(), // Light text for overlays
        },
        status_line: StatusLineColors {
            bg: "#3b4048".to_string(),  // Status line background (darker)
            text: "#ffffff".to_string(), // White text for status line
        },
    }
}