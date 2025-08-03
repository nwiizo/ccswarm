//! Visual error diagrams and ASCII art for better error understanding

use colored::Colorize;

/// Common error diagram template structure
#[derive(Debug)]
pub struct DiagramConfig {
    pub title: String,
    pub title_color: colored::Color,
    pub sections: Vec<DiagramSection>,
    pub footer: Option<String>,
}

#[derive(Debug)]
pub struct DiagramSection {
    pub content: String,
    pub highlights: Vec<(String, colored::Color)>,
    pub section_type: SectionType,
}

#[derive(Debug)]
pub enum SectionType {
    Text,
    BoxDiagram,
    FlowDiagram,
    FileContent,
    NumberedSteps,
}

/// Template components for building diagrams
pub struct DiagramComponents {
    // Box drawing characters
    pub box_top_left: &'static str,
    pub box_top_right: &'static str,
    pub box_bottom_left: &'static str,
    pub box_bottom_right: &'static str,
    pub box_horizontal: &'static str,
    pub box_vertical: &'static str,
    pub box_cross: &'static str,
    pub box_t_down: &'static str,
    pub box_t_up: &'static str,
    pub box_t_right: &'static str,
    pub box_t_left: &'static str,

    // Arrows
    pub arrow_right: &'static str,
    pub arrow_left: &'static str,
    pub arrow_down: &'static str,
    pub arrow_up: &'static str,
    pub arrow_long_right: &'static str,

    // Status symbols
    pub check_mark: &'static str,
    pub cross_mark: &'static str,
    pub warning: &'static str,
    pub question: &'static str,
    pub lightning: &'static str,
}

impl Default for DiagramComponents {
    fn default() -> Self {
        Self {
            // Box drawing
            box_top_left: "┌",
            box_top_right: "┐",
            box_bottom_left: "└",
            box_bottom_right: "┘",
            box_horizontal: "─",
            box_vertical: "│",
            box_cross: "┼",
            box_t_down: "┬",
            box_t_up: "┴",
            box_t_right: "├",
            box_t_left: "┤",

            // Arrows
            arrow_right: "▶",
            arrow_left: "◄",
            arrow_down: "▼",
            arrow_up: "▲",
            arrow_long_right: "──▶",

            // Status
            check_mark: "✓",
            cross_mark: "✗",
            warning: "⚠",
            question: "?",
            lightning: "⚡",
        }
    }
}

/// Visual error diagrams for common scenarios
pub struct ErrorDiagrams;

impl ErrorDiagrams {
    /// Generic diagram builder with enhanced replacement support
    fn build_diagram(config: DiagramConfig) -> String {
        let mut result = String::new();
        let components = DiagramComponents::default();

        // Add title
        result.push_str(&format!(
            "\n    {}\n",
            config.title.color(config.title_color).bold()
        ));

        // Add sections
        for section in config.sections {
            let mut section_content = section.content;

            // Apply component replacements
            section_content = Self::replace_components(&section_content, &components);

            // Apply highlights - enhanced to handle both {text} and direct replacements
            for (placeholder, color) in section.highlights {
                // Handle both {placeholder} and direct placeholder patterns
                if placeholder.starts_with('{') && placeholder.ends_with('}') {
                    // This is a placeholder to be replaced with colored text
                    let text = placeholder.trim_start_matches('{').trim_end_matches('}');
                    section_content =
                        section_content.replace(&placeholder, &text.color(color).to_string());
                } else {
                    // This is text to be highlighted
                    section_content = section_content.replace(
                        &format!("{{{}}}", placeholder),
                        &placeholder.color(color).to_string(),
                    );
                }
            }

            // Apply section-specific formatting
            match section.section_type {
                SectionType::BoxDiagram | SectionType::FlowDiagram => {
                    // Already formatted
                    result.push_str(&section_content);
                }
                SectionType::FileContent => {
                    // Add indentation for file content
                    for line in section_content.lines() {
                        result.push_str(&format!("    {}\n", line));
                    }
                }
                SectionType::NumberedSteps => {
                    // Ensure proper spacing for steps
                    result.push_str(&section_content);
                }
                SectionType::Text => {
                    result.push_str(&section_content);
                }
            }
        }

        // Add footer if present
        if let Some(footer) = config.footer {
            result.push_str(&format!("\n    {}\n", footer));
        }

        result
    }

    /// Replace component placeholders with actual characters
    fn replace_components(content: &str, components: &DiagramComponents) -> String {
        content
            .replace("{{BOX_TL}}", components.box_top_left)
            .replace("{{BOX_TR}}", components.box_top_right)
            .replace("{{BOX_BL}}", components.box_bottom_left)
            .replace("{{BOX_BR}}", components.box_bottom_right)
            .replace("{{BOX_H}}", components.box_horizontal)
            .replace("{{BOX_V}}", components.box_vertical)
            .replace("{{BOX_CROSS}}", components.box_cross)
            .replace("{{BOX_TD}}", components.box_t_down)
            .replace("{{BOX_TU}}", components.box_t_up)
            .replace("{{BOX_TR_T}}", components.box_t_right)
            .replace("{{BOX_TL_T}}", components.box_t_left)
            .replace("{{ARROW_R}}", components.arrow_right)
            .replace("{{ARROW_L}}", components.arrow_left)
            .replace("{{ARROW_D}}", components.arrow_down)
            .replace("{{ARROW_U}}", components.arrow_up)
            .replace("{{ARROW_LR}}", components.arrow_long_right)
            .replace("{{CHECK}}", components.check_mark)
            .replace("{{CROSS}}", components.cross_mark)
            .replace("{{WARN}}", components.warning)
            .replace("{{QUESTION}}", components.question)
            .replace("{{LIGHTNING}}", components.lightning)
    }

    /// Helper to create a box with title
    #[allow(dead_code)]
    fn create_box(title: &str, width: usize) -> String {
        let components = DiagramComponents::default();
        let padding = width.saturating_sub(title.len() + 2);
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;

        format!(
            "{}{}{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}",
            components.box_top_left,
            components.box_horizontal.repeat(left_pad),
            title,
            components.box_horizontal.repeat(right_pad),
            components.box_top_right,
            components.box_vertical,
            " ".repeat(width - 2),
            components.box_vertical,
            components.box_vertical,
            " ".repeat(width - 2),
            components.box_vertical,
            components.box_bottom_left,
            components.box_horizontal.repeat(width - 2),
            components.box_bottom_right
        )
    }
    /// Network connectivity diagram
    pub fn network_error() -> String {
        let config = DiagramConfig {
            title: "Network Connection Error:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}       {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}       {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}   {Your} {{BOX_V}}  ❌  {{BOX_V}}  {Internet} {{BOX_V}}  ❌  {{BOX_V}}   {Claude} {{BOX_V}}
    {{BOX_V}}  Computer   {{BOX_V}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_R}}{{BOX_V}}   Network    {{BOX_V}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_R}}{{BOX_V}}     API     {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}       {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}       {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
           {{BOX_V}}                      {{BOX_V}}                       {{BOX_V}}
           {{BOX_V}}                      {{BOX_V}}                       {{BOX_V}}
           {{ARROW_D}}                      {{ARROW_D}}                       {{ARROW_D}}
      {{CHECK}} Check              {{CROSS}} Failed            ? Unknown
      API Key            Connection           Server Status
                          Issue?                 Down?
"#.to_string(),
                    highlights: vec![
                        ("Your".to_string(), colored::Color::BrightWhite),
                        ("Internet".to_string(), colored::Color::BrightWhite),
                        ("Claude".to_string(), colored::Color::BrightWhite),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: None,
        };

        Self::build_diagram(config)
    }

    /// Session lifecycle diagram
    pub fn session_error() -> String {
        let config = DiagramConfig {
            title: "Session State Diagram:".to_string(),
            title_color: colored::Color::BrightCyan,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}                    Session Lifecycle                     {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    {Created} {{ARROW_LR}} {Active} {{ARROW_LR}} {Idle} {{ARROW_LR}} {Terminated}
         {{BOX_V}}         {{BOX_V}}         {{BOX_V}}         {{BOX_V}}
         {{ARROW_D}}         {{ARROW_D}}         {{ARROW_D}}         {{ARROW_D}}
      {New}    {Working}    {Waiting}    {Closed}
"#.to_string(),
                    highlights: vec![
                        ("Created".to_string(), colored::Color::Green),
                        ("Active".to_string(), colored::Color::BrightGreen),
                        ("Idle".to_string(), colored::Color::Yellow),
                        ("Terminated".to_string(), colored::Color::Red),
                        ("New".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Working".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Waiting".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Closed".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                    ],
                    section_type: SectionType::FlowDiagram,
                },
            ],
            footer: Some("Your session might be in any of these states.\n    Use 'ccswarm session list' to check current sessions.".to_string()),
        };

        Self::build_diagram(config)
    }

    /// Git worktree visualization
    pub fn git_worktree_error() -> String {
        let config = DiagramConfig {
            title: "Git Worktree Structure:".to_string(),
            title_color: colored::Color::BrightYellow,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}                     Main Repository                      {{BOX_V}}
    {{BOX_V}}  {master_branch} {{ARROW_L}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TL}}               {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TD}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_CROSS}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
                      {{BOX_V}}               {{BOX_V}}
         {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}      {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
         {{BOX_V}} {frontend_agent} {{BOX_V}}      {{BOX_V}} {backend_agent} {{BOX_V}}
         {{BOX_V}}   frontend     {{BOX_V}}      {{BOX_V}}    backend      {{BOX_V}}
         {{BOX_V}}  (worktree)    {{BOX_V}}      {{BOX_V}}  (worktree)     {{BOX_V}}
         {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}      {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
              {active_status}                    {conflict_status}
"#.to_string(),
                    highlights: vec![
                        ("master_branch".to_string(), colored::Color::BrightWhite),
                        ("frontend_agent".to_string(), colored::Color::Cyan),
                        ("backend_agent".to_string(), colored::Color::Magenta),
                        ("active_status".to_string(), colored::Color::Green),
                        ("conflict_status".to_string(), colored::Color::Red),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: Some("Each agent works in its own isolated worktree.\n    Conflicts occur when a branch is already checked out.".to_string()),
        };

        // Replace the placeholder text with actual symbols
        let mut result = Self::build_diagram(config);
        result = result.replace("{master_branch}", "master branch");
        result = result.replace("{frontend_agent}", "Agent: Frontend");
        result = result.replace("{backend_agent}", "Agent: Backend");
        result = result.replace("{active_status}", "✓ Active");
        result = result.replace("{conflict_status}", "✗ Conflict");

        result
    }

    /// Permission hierarchy
    pub fn permission_error() -> String {
        let config = DiagramConfig {
            title: "Permission Structure:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}                  File Permissions                        {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    Owner    Group    Others
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}  {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}  {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {owner_r} {{BOX_V}}  {{BOX_V}} {group_r} {{BOX_V}}  {{BOX_V}} {other_r} {{BOX_V}}    {r_legend} = Read
    {{BOX_V}} {owner_w} {{BOX_V}}  {{BOX_V}} {group_w} {{BOX_V}}  {{BOX_V}} {other_w} {{BOX_V}}    {w_legend} = Write  
    {{BOX_V}} {owner_x} {{BOX_V}}  {{BOX_V}} {group_x} {{BOX_V}}  {{BOX_V}} {other_x} {{BOX_V}}    {x_legend} = Execute
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}  {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}  {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
      {you_label}      {team_label}      {others_label}
"#.to_string(),
                    highlights: vec![
                        ("owner_r".to_string(), colored::Color::Green),
                        ("group_r".to_string(), colored::Color::Green),
                        ("other_r".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("r_legend".to_string(), colored::Color::BrightWhite),
                        ("owner_w".to_string(), colored::Color::Green),
                        ("group_w".to_string(), colored::Color::Yellow),
                        ("other_w".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("w_legend".to_string(), colored::Color::BrightWhite),
                        ("owner_x".to_string(), colored::Color::Green),
                        ("group_x".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("other_x".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("x_legend".to_string(), colored::Color::BrightWhite),
                        ("you_label".to_string(), colored::Color::BrightGreen),
                        ("team_label".to_string(), colored::Color::Yellow),
                        ("others_label".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: Some("Current user needs appropriate permissions to access files.".to_string()),
        };

        // Replace the placeholder text with actual values
        let mut result = Self::build_diagram(config);
        result = result.replace("{owner_r}", "r");
        result = result.replace("{group_r}", "r");
        result = result.replace("{other_r}", "r");
        result = result.replace("{r_legend}", "r");
        result = result.replace("{owner_w}", "w");
        result = result.replace("{group_w}", "w");
        result = result.replace("{other_w}", "-");
        result = result.replace("{w_legend}", "w");
        result = result.replace("{owner_x}", "x");
        result = result.replace("{group_x}", "-");
        result = result.replace("{other_x}", "-");
        result = result.replace("{x_legend}", "x");
        result = result.replace("{you_label}", "You");
        result = result.replace("{team_label}", "Team");
        result = result.replace("{others_label}", "Others");

        result
    }

    /// Configuration file structure
    pub fn config_error() -> String {
        let config = DiagramConfig {
            title: "Configuration Structure:".to_string(),
            title_color: colored::Color::BrightCyan,
            sections: vec![
                DiagramSection {
                    content: r#"{{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
{{BOX_V}}                  ccswarm.json                            {{BOX_V}}
{{BOX_TR_T}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TL_T}}
{{BOX_V}} {                                                        {{BOX_V}}
{{BOX_V}}   "project": {                                           {{BOX_V}}
{{BOX_V}}     "name": "{project_name}",                               {{BOX_V}}
{{BOX_V}}     "description": "{project_desc}"                 {{BOX_V}}
{{BOX_V}}   },                                                     {{BOX_V}}
{{BOX_V}}   "agents": [                                            {{BOX_V}}
{{BOX_V}}     {                                                    {{BOX_V}}
{{BOX_V}}       "name": "{agent_name}",                            {{BOX_V}}
{{BOX_V}}       "role": "{agent_role}",                            {{BOX_V}}
{{BOX_V}}       "provider": "{provider}"                      {{BOX_V}}
{{BOX_V}}     }                                                    {{BOX_V}}
{{BOX_V}}   ]                                                      {{BOX_V}}
{{BOX_V}} }                                                        {{BOX_V}}
{{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
"#.to_string(),
                    highlights: vec![
                        ("project_name".to_string(), colored::Color::BrightGreen),
                        ("project_desc".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("agent_name".to_string(), colored::Color::BrightYellow),
                        ("agent_role".to_string(), colored::Color::BrightWhite),
                        ("provider".to_string(), colored::Color::BrightMagenta),
                    ],
                    section_type: SectionType::FileContent,
                },
            ],
            footer: Some("This configuration file is required to run ccswarm.\n    Use 'ccswarm init' to create it automatically.".to_string()),
        };

        // Replace the placeholder text with actual values
        let mut result = Self::build_diagram(config);
        result = result.replace("{project_name}", "MyProject");
        result = result.replace("{project_desc}", "AI orchestration project");
        result = result.replace("{agent_name}", "frontend");
        result = result.replace("{agent_role}", "Frontend");
        result = result.replace("{provider}", "claude_code");

        result
    }

    /// Task flow diagram
    pub fn task_error() -> String {
        let config = DiagramConfig {
            title: "Task Processing Flow:".to_string(),
            title_color: colored::Color::BrightYellow,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}      {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}      {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}      {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}  {Input} {{BOX_V}} {{ARROW_LR}} {{BOX_V}} {Parse} {{BOX_V}} {{ARROW_LR}} {{BOX_V}} {Assign} {{BOX_V}} {{ARROW_LR}} {{BOX_V}} {Execute} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}      {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}      {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}      {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
         {{BOX_V}}                 {{BOX_V}}                 {{BOX_V}}                 {{BOX_V}}
         {{ARROW_D}}                 {{ARROW_D}}                 {{ARROW_D}}                 {{ARROW_D}}
    {Description}         {Priority}      {Agent_Role}       {Result}
    
    Task Format: "{desc_format} [{priority_format}] [{type_format}]"
    
    Examples:
    • "Create user authentication system [high] [feature]"
    • "Fix login bug [urgent] [bugfix]"
    • "Add unit tests for API [medium] [test]"
"#.to_string(),
                    highlights: vec![
                        ("Input".to_string(), colored::Color::BrightWhite),
                        ("Parse".to_string(), colored::Color::BrightCyan),
                        ("Assign".to_string(), colored::Color::BrightGreen),
                        ("Execute".to_string(), colored::Color::BrightMagenta),
                        ("Description".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Priority".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Agent_Role".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Result".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("desc_format".to_string(), colored::Color::BrightWhite),
                        ("priority_format".to_string(), colored::Color::Yellow),
                        ("type_format".to_string(), colored::Color::Cyan),
                    ],
                    section_type: SectionType::FlowDiagram,
                },
            ],
            footer: None,
        };

        // Replace the placeholder text with actual values
        let mut result = Self::build_diagram(config);
        result = result.replace("{Agent_Role}", "Agent Role");
        result = result.replace("{desc_format}", "description");
        result = result.replace("{priority_format}", "priority");
        result = result.replace("{type_format}", "type");

        result
    }

    /// API key flow
    pub fn api_key_error() -> String {
        let config = DiagramConfig {
            title: "API Key Configuration:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}                  API Key Setup                           {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    1. {Visit} {{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_R}} https://console.anthropic.com
       {{BOX_BL}}{{BOX_H}}{{ARROW_R}} Sign up / Log in
    
    2. {Navigate} {{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_R}} API Keys section
       {{BOX_BL}}{{BOX_H}}{{ARROW_R}} Create new key
    
    3. {Generate} {{BOX_H}}{{BOX_H}}{{ARROW_R}} Copy key: sk-ant-api03-...
       {{BOX_BL}}{{BOX_H}}{{ARROW_R}} Keep it secret!
    
    4. {Configure} {{BOX_H}}{{ARROW_R}} Export in terminal:
       {export_cmd}
    
    5. {Persist} {{BOX_H}}{{BOX_H}}{{ARROW_R}} Add to .env file:
       {env_cmd}
"#.to_string(),
                    highlights: vec![
                        ("Visit".to_string(), colored::Color::BrightWhite),
                        ("Navigate".to_string(), colored::Color::BrightWhite),
                        ("Generate".to_string(), colored::Color::BrightWhite),
                        ("Configure".to_string(), colored::Color::BrightWhite),
                        ("export_cmd".to_string(), colored::Color::BrightGreen),
                        ("Persist".to_string(), colored::Color::BrightWhite),
                        ("env_cmd".to_string(), colored::Color::BrightGreen),
                    ],
                    section_type: SectionType::NumberedSteps,
                },
            ],
            footer: None,
        };

        // Replace the placeholder text with actual commands
        let mut result = Self::build_diagram(config);
        result = result.replace("{export_cmd}", "export ANTHROPIC_API_KEY=your-key-here");
        result = result.replace("{env_cmd}", "echo 'ANTHROPIC_API_KEY=your-key' >> .env");

        result
    }

    /// Session pool exhausted diagram
    pub fn session_pool_exhausted(role: &str) -> String {
        let config = DiagramConfig {
            title: format!("Session Pool Exhausted for Role: {}", role),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}                    Session Pool Status                      {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    Pool: {role_name}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} Session 1: {busy1} {{BOX_V}}
    {{BOX_V}} Session 2: {busy2} {{BOX_V}}
    {{BOX_V}} Session 3: {busy3} {{BOX_V}}
    {{BOX_V}} Session 4: {busy4} {{BOX_V}}
    {{BOX_V}} Session 5: {busy5} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    {{CROSS}} All sessions busy!
    {{WARN}} No available sessions
    
    Recovery Actions:
    1. Wait for session to free up
    2. Increase pool size limit
    3. Use fallback agent
"#.to_string(),
                    highlights: vec![
                        ("role_name".to_string(), colored::Color::BrightYellow),
                        ("busy1".to_string(), colored::Color::Red),
                        ("busy2".to_string(), colored::Color::Red),
                        ("busy3".to_string(), colored::Color::Red),
                        ("busy4".to_string(), colored::Color::Red),
                        ("busy5".to_string(), colored::Color::Red),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: None,
        };

        let mut result = Self::build_diagram(config);
        result = result.replace("{role_name}", role);
        result = result.replace("{busy1}", "BUSY");
        result = result.replace("{busy2}", "BUSY");
        result = result.replace("{busy3}", "BUSY");
        result = result.replace("{busy4}", "BUSY");
        result = result.replace("{busy5}", "BUSY");
        
        result
    }

    /// Circular dependency diagram
    pub fn circular_dependency(task_ids: &[String]) -> String {
        let config = DiagramConfig {
            title: "Circular Dependency Detected:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {task1} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
            {{ARROW_D}}
            depends on
            {{ARROW_D}}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {task2} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
            {{ARROW_D}}
            depends on
            {{ARROW_D}}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {task3} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
            {{ARROW_D}}
            depends on
            {{ARROW_U}}
    
    {{CROSS}} Circular dependency!
    Tasks cannot be executed.
"#.to_string(),
                    highlights: vec![
                        ("task1".to_string(), colored::Color::BrightCyan),
                        ("task2".to_string(), colored::Color::BrightCyan),
                        ("task3".to_string(), colored::Color::BrightCyan),
                    ],
                    section_type: SectionType::FlowDiagram,
                },
            ],
            footer: Some("Break the cycle by removing one dependency.".to_string()),
        };

        let mut result = Self::build_diagram(config);
        if task_ids.len() >= 3 {
            result = result.replace("{task1}", &task_ids[0]);
            result = result.replace("{task2}", &task_ids[1]);
            result = result.replace("{task3}", &task_ids[2]);
        } else {
            result = result.replace("{task1}", "Task A");
            result = result.replace("{task2}", "Task B");
            result = result.replace("{task3}", "Task C");
        }
        
        result
    }

    /// Role violation diagram
    pub fn role_violation(agent: &str, forbidden_action: &str) -> String {
        let config = DiagramConfig {
            title: "Agent Role Violation:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {agent_name} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
            {{ARROW_D}}
            attempted
            {{ARROW_D}}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {forbidden} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    {{CROSS}} FORBIDDEN!
    
    Allowed actions for {agent_role}:
    {{CHECK}} UI/UX development
    {{CHECK}} Component creation
    {{CHECK}} Styling and CSS
    
    Not allowed:
    {{CROSS}} Database operations
    {{CROSS}} API modifications
    {{CROSS}} Infrastructure changes
"#.to_string(),
                    highlights: vec![
                        ("agent_name".to_string(), colored::Color::BrightYellow),
                        ("forbidden".to_string(), colored::Color::BrightRed),
                        ("agent_role".to_string(), colored::Color::BrightYellow),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: Some("Agents must stay within their designated roles.".to_string()),
        };

        let mut result = Self::build_diagram(config);
        result = result.replace("{agent_name}", agent);
        result = result.replace("{forbidden}", forbidden_action);
        result = result.replace("{agent_role}", &agent.replace("-agent", ""));
        
        result
    }

    /// Connection failed diagram
    pub fn connection_failed(endpoint: &str, reason: &str) -> String {
        let config = DiagramConfig {
            title: "Network Connection Failed:".to_string(),
            title_color: colored::Color::BrightRed,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}       {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}   ccswarm   {{BOX_V}}  ❌  {{BOX_V}} {endpoint_name} {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}       {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
    
    Error: {error_reason}
    
    Possible causes:
    {{WARN}} Network connectivity issue
    {{WARN}} Service unavailable
    {{WARN}} Authentication failure
    {{WARN}} Firewall blocking connection
    
    Try:
    1. Check your internet connection
    2. Verify API credentials
    3. Check service status
    4. Review firewall settings
"#.to_string(),
                    highlights: vec![
                        ("endpoint_name".to_string(), colored::Color::BrightWhite),
                        ("error_reason".to_string(), colored::Color::BrightRed),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: None,
        };

        let mut result = Self::build_diagram(config);
        result = result.replace("{endpoint_name}", endpoint);
        result = result.replace("{error_reason}", reason);
        
        result
    }

    /// Agent communication flow
    pub fn agent_error() -> String {
        let config = DiagramConfig {
            title: "Agent Communication:".to_string(),
            title_color: colored::Color::BrightCyan,
            sections: vec![
                DiagramSection {
                    content: r#"    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}} {Master_Claude} {{BOX_V}}
    {{BOX_V}}  Orchestrator {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TD}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
            {{BOX_V}} {Delegates_tasks}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}          Task Assignment                   {{BOX_V}}
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TD}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TD}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TD}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
         {{BOX_V}}          {{BOX_V}}          {{BOX_V}}
    {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}} {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}} {{BOX_TL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{ARROW_D}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_TR}}
    {{BOX_V}}{Frontend}{{BOX_V}} {{BOX_V}}{Backend}{{BOX_V}} {{BOX_V}}{DevOps} {{BOX_V}}  {error_arrow} 
    {{BOX_V}}Agent   {{BOX_V}} {{BOX_V}}Agent   {{BOX_V}} {{BOX_V}}Agent   {{BOX_V}}  Agent Busy!
    {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}} {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}} {{BOX_BL}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_H}}{{BOX_BR}}
       {ready1}        {ready2}        {busy}
"#.to_string(),
                    highlights: vec![
                        ("Master_Claude".to_string(), colored::Color::BrightYellow),
                        ("Delegates_tasks".to_string(), colored::Color::TrueColor { r: 128, g: 128, b: 128 }),
                        ("Frontend".to_string(), colored::Color::Cyan),
                        ("Backend".to_string(), colored::Color::Magenta),
                        ("DevOps".to_string(), colored::Color::Green),
                        ("error_arrow".to_string(), colored::Color::Red),
                        ("ready1".to_string(), colored::Color::Green),
                        ("ready2".to_string(), colored::Color::Green),
                        ("busy".to_string(), colored::Color::Yellow),
                    ],
                    section_type: SectionType::BoxDiagram,
                },
            ],
            footer: Some("Agents can only handle one task at a time.\n    Use 'ccswarm agent status' to check availability.".to_string()),
        };

        // Replace the placeholder text with actual values
        let mut result = Self::build_diagram(config);
        result = result.replace("{Master_Claude}", "Master Claude");
        result = result.replace("{Delegates_tasks}", "Delegates tasks");
        result = result.replace("{error_arrow}", "←── ❌");
        result = result.replace("{ready1}", "✓ Ready");
        result = result.replace("{ready2}", "✓ Ready");
        result = result.replace("{busy}", "⚡ Busy");

        result
    }
}

/// Helper to display a diagram with proper formatting
pub fn show_diagram(diagram: String) {
    println!();
    for line in diagram.lines() {
        println!("{}", line);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagrams_render() {
        // Just ensure they don't panic
        let _ = ErrorDiagrams::network_error();
        let _ = ErrorDiagrams::session_error();
        let _ = ErrorDiagrams::git_worktree_error();
        let _ = ErrorDiagrams::permission_error();
        let _ = ErrorDiagrams::config_error();
        let _ = ErrorDiagrams::task_error();
        let _ = ErrorDiagrams::api_key_error();
        let _ = ErrorDiagrams::agent_error();
    }

    #[test]
    fn test_template_engine() {
        // Test basic template functionality
        let config = DiagramConfig {
            title: "Test Title".to_string(),
            title_color: colored::Color::Red,
            sections: vec![DiagramSection {
                content: "{{BOX_TL}}{{BOX_H}}{{BOX_TR}} {test}".to_string(),
                highlights: vec![("test".to_string(), colored::Color::Green)],
                section_type: SectionType::Text,
            }],
            footer: Some("Test footer".to_string()),
        };

        let result = ErrorDiagrams::build_diagram(config);
        assert!(result.contains("┌"));
        assert!(result.contains("─"));
        assert!(result.contains("┐"));
        assert!(result.contains("Test footer"));
    }

    #[test]
    fn test_component_replacement() {
        let components = DiagramComponents::default();
        let content = "{{BOX_TL}}{{ARROW_R}}{{CHECK}}";
        let result = ErrorDiagrams::replace_components(content, &components);

        assert_eq!(result, "┌▶✓");
    }

    #[test]
    fn test_all_diagrams_use_template_engine() {
        // Ensure all methods produce valid output with box characters
        let diagrams = vec![
            ErrorDiagrams::network_error(),
            ErrorDiagrams::session_error(),
            ErrorDiagrams::git_worktree_error(),
            ErrorDiagrams::permission_error(),
            ErrorDiagrams::config_error(),
            ErrorDiagrams::task_error(),
            ErrorDiagrams::api_key_error(),
            ErrorDiagrams::agent_error(),
        ];

        for diagram in diagrams {
            // Check that box drawing characters are present
            assert!(diagram.contains("┌") || diagram.contains("┏"));
            assert!(diagram.contains("─") || diagram.contains("━"));
            assert!(diagram.contains("│") || diagram.contains("┃"));
        }
    }

    #[test]
    fn test_section_types() {
        // Test different section type formatting
        let text_section = DiagramSection {
            content: "Simple text".to_string(),
            highlights: vec![],
            section_type: SectionType::Text,
        };

        let file_content_section = DiagramSection {
            content: "{\n  \"key\": \"value\"\n}".to_string(),
            highlights: vec![],
            section_type: SectionType::FileContent,
        };

        let config = DiagramConfig {
            title: "Test".to_string(),
            title_color: colored::Color::White,
            sections: vec![text_section, file_content_section],
            footer: None,
        };

        let result = ErrorDiagrams::build_diagram(config);

        // FileContent should be indented
        assert!(result.contains("    {"));
        assert!(result.contains("      \"key\": \"value\""));
        assert!(result.contains("    }"));
    }
}
