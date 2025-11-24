
use crate::pandoc::{Block, Pandoc};
use crate::pandoc::ASTContext;
use quarto_source_map::{FileId, SourceInfo};
use std::io::{self, Write};

struct RBlockRange {
    start: usize,
    end: usize,
}

pub fn write(pandoc: &Pandoc, context: &ASTContext, writer: &mut impl Write) -> io::Result<()> {
    // Assume the primary file is the first one in the context
    if let Some(file) = context.source_context.get_file(FileId(0)) {
        if let Some(content) = &file.content {
            // Start with all lines commented out
            let mut output_lines: Vec<String> =
                content.lines().map(|_line| "#".to_string()).collect();

            // Now, "uncomment" the R code blocks
            for block in &pandoc.blocks {
                if let Block::CodeBlock(code_block) = block {
                    let is_r = code_block.attr.1.iter().any(|c| c == "r" || c == "{r}");
                    if is_r {
                        let (start, _) = get_block_lines(block, context);
                        // The parser gives us the code content, which starts on the line after the opening fence
                        let code_start_line = start + 1;

                        for (i, line) in code_block.text.lines().enumerate() {
                            let line_idx = code_start_line + i;
                            if line_idx < output_lines.len() {
                                output_lines[line_idx] = line.to_string();
                            }
                        }
                    }
                }
            }

            // Write the result
            for line in output_lines {
                writeln!(writer, "{}", line)?;
            }
        }
    }

    Ok(())
}

fn get_block_lines(block: &Block, context: &ASTContext) -> (usize, usize) {
    let source_info = match block {
        Block::Plain(b) => &b.source_info,
        Block::Paragraph(b) => &b.source_info,
        Block::LineBlock(b) => &b.source_info,
        Block::CodeBlock(b) => &b.source_info,
        Block::RawBlock(b) => &b.source_info,
        Block::BlockQuote(b) => &b.source_info,
        Block::OrderedList(b) => &b.source_info,
        Block::BulletList(b) => &b.source_info,
        Block::DefinitionList(b) => &b.source_info,
        Block::Header(b) => &b.source_info,
        Block::HorizontalRule(b) => &b.source_info,
        Block::Table(b) => &b.source_info,
        Block::Figure(b) => &b.source_info,
        Block::Div(b) => &b.source_info,
        _ => return (0, 0),
    };

    if let Some((file_id, start_offset, end_offset)) = get_original_source_info(source_info) {
        if let Some(file) = context.source_context.get_file(file_id) {
            if let Some(file_info) = &file.file_info {
                if let Some(content) = &file.content {
                    let start_loc = file_info.offset_to_location(start_offset, content);
                    let end_loc = file_info.offset_to_location(end_offset, content);
                    // Return 0-indexed row numbers
                    return (start_loc.unwrap().row, end_loc.unwrap().row);
                }
            }
        }
    }
    (0, 0)
}

fn get_original_source_info(source_info: &SourceInfo) -> Option<(FileId, usize, usize)> {
    match source_info {
        SourceInfo::Original {
            file_id,
            start_offset,
            end_offset,
        } => Some((*file_id, *start_offset, *end_offset)),
        SourceInfo::Substring { parent, .. } => get_original_source_info(parent),
        SourceInfo::Concat { pieces } => {
            if let Some(first_piece) = pieces.first() {
                get_original_source_info(&first_piece.source_info)
            } else {
                None
            }
        }
    }
}
