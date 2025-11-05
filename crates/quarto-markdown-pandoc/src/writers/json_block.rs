/*
 * json_block.rs
 * Copyright (c) 2025 Posit, PBC
 */

use crate::pandoc::{Block, Pandoc};
use crate::pandoc::ASTContext;
use quarto_source_map::{FileId, SourceInfo};
use serde_json::json;
use std::io::{self, Write};

pub fn write(
    pandoc: &Pandoc,
    context: &ASTContext,
    line_number: usize,
    writer: &mut impl Write,
) -> io::Result<()> {
    let mut result = json!({
        "code": "",
        "start_line": 0
    });

    // The line number from the command line is 1-based, but our internal line
    // numbers are 0-based.
    let line_number = line_number.saturating_sub(1);

    for block in &pandoc.blocks {
        if let Block::CodeBlock(code_block) = block {
            let (start, end) = get_block_lines(block, context);
            if line_number >= start && line_number <= end {
                result = json!({
                    "code": code_block.text,
                    "start_line": start + 1,
                });
                break;
            }
        }
    }

    serde_json::to_writer(writer, &result)?;
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
                let start_loc = file_info.offset_to_location(start_offset);
                let end_loc = file_info.offset_to_location(end_offset);
                // Return 0-indexed row numbers
                return (start_loc.unwrap().row, end_loc.unwrap().row);
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
