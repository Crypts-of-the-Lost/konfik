// SPDX-License-Identifier: MIT
// Copyright (c) 2025 kingananas20

use syn::{Field, Type, TypePath};

/// Analysis result for a field
#[expect(clippy::struct_excessive_bools)]
pub struct FieldAnalysis {
    pub skip: bool,
    pub required: bool,
    pub has_default: bool,
    pub nested: bool,
}

/// Analyze a field to determine its requirements
pub fn analyze_field(field: &Field) -> Result<FieldAnalysis, syn::Error> {
    let mut analysis = FieldAnalysis {
        skip: false,
        required: false,
        has_default: false,
        nested: false,
    };

    for attr in &field.attrs {
        // handle #[konfik(...)]
        if attr.path().is_ident("konfik") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    analysis.skip = true;
                } else if meta.path.is_ident("nested") {
                    analysis.nested = true;
                }
                Ok(())
            })?;
        }

        // handle #[command(...)]
        if attr.path().is_ident("command") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("flatten") {
                    analysis.nested = true;
                }
                Ok(())
            })?;
        }

        // handle #[serde(...)]
        if attr.path().is_ident("serde") {
            // parse_nested_meta calls our closure for each comma-separated item inside the `(...)`
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    analysis.skip = true;
                } else if meta.path.is_ident("default") {
                    // `default` can appear as `default` or `default = "..."`; either way we mark has_default
                    analysis.has_default = true;
                }
                // return Ok(()) to continue parsing other nested items
                Ok(())
            })?;
        }
    }

    // keep your original semantics: required if not Option<T> and no default
    analysis.required = !is_option_type(&field.ty) && !analysis.has_default;

    Ok(analysis)
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }
    false
}
