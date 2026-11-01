use crate::document::{
    Blob, Color, Effect, FigNode, Gradient, GradientStop, Guid, Matrix, Paint, Size, TextData,
    VectorGeometry,
};
use crate::error::{FigError, Result};
use crate::limits::Limits;

use super::cursor::{Cursor, CursorError};
use super::schema::{
    Def, DefKind, Field, Schema, TYPE_BOOL, TYPE_BYTE, TYPE_FLOAT, TYPE_INT, TYPE_INT64,
    TYPE_STRING, TYPE_UINT, TYPE_UINT64,
};

/// Decode the root `Message` into nodes and blob ranges.
pub(crate) fn decode_message(
    schema: &Schema,
    data: &[u8],
    limits: &Limits,
) -> Result<(Vec<FigNode>, Vec<Blob>)> {
    let root_index = schema
        .index_of("Message")
        .ok_or_else(|| FigError::Schema("schema has no Message definition".into()))?;
    let root = schema
        .def(root_index)
        .ok_or_else(|| FigError::Schema("Message definition is missing".into()))?;
    if root.kind != DefKind::Message {
        return Err(FigError::Schema("Message is not a Kiwi message".into()));
    }

    let mut cursor = Cursor::new(data);
    let mut nodes: Option<Vec<FigNode>> = None;
    let mut blobs: Vec<Blob> = Vec::new();
    let mut seen_blobs = false;

    loop {
        let field_id = read_u32(&mut cursor, "top-level field id")?;
        if field_id == 0 {
            break;
        }
        let field = root.field_by_value(field_id).ok_or_else(|| {
            FigError::Decode(format!(
                "Message contains unknown field id {field_id} (schema has {} fields)",
                root.fields.len()
            ))
        })?;

        match field.name.as_str() {
            "nodeChanges" => {
                if !field.is_array {
                    return Err(FigError::Decode(
                        "Message.nodeChanges is not an array".into(),
                    ));
                }
                if nodes.is_some() {
                    return Err(FigError::Decode(
                        "Message.nodeChanges appears more than once".into(),
                    ));
                }
                nodes = Some(decode_nodes(&mut cursor, schema, field, limits)?);
            }
            "blobs" => {
                if !field.is_array {
                    return Err(FigError::Decode("Message.blobs is not an array".into()));
                }
                if seen_blobs {
                    return Err(FigError::Decode(
                        "Message.blobs appears more than once".into(),
                    ));
                }
                blobs = decode_blobs(&mut cursor, schema, field, limits)?;
                seen_blobs = true;
            }
            _ => skip_field(&mut cursor, schema, field, limits, 0)?,
        }
    }

    if !cursor.is_empty() {
        return Err(FigError::Decode(format!(
            "Message has {} trailing bytes",
            cursor.remaining()
        )));
    }

    let nodes = nodes.ok_or_else(|| FigError::Decode("Message has no nodeChanges".into()))?;
    Ok((nodes, blobs))
}

fn decode_nodes(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
) -> Result<Vec<FigNode>> {
    let count = read_count(cursor, limits.max_nodes, "node count")?;
    let mut nodes = Vec::with_capacity(count.min(10_000));
    for _ in 0..count {
        nodes.push(decode_node(cursor, schema, field.type_id, limits)?);
    }
    Ok(nodes)
}

fn decode_node(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    def_index: i32,
    limits: &Limits,
) -> Result<FigNode> {
    let def = schema
        .def(def_index as usize)
        .ok_or_else(|| FigError::Decode(format!("node element type {def_index} does not exist")))?;

    let mut node = FigNode {
        guid: Guid::ZERO,
        parent: None,
        position: None,
        kind: "UNKNOWN".to_owned(),
        kind_raw: None,
        name: String::new(),
        phase: None,
        visible: None,
        size: None,
        transform: None,
        internal_only: false,
        removed: false,
        opacity: None,
        fills: Vec::new(),
        strokes: Vec::new(),
        stroke_weight: None,
        stroke_align: None,
        corner_radius: None,
        corner_radii: None,
        resize_to_fit: None,
        text: None,
        effects: Vec::new(),
        fill_geometry: Vec::new(),
        stroke_geometry: Vec::new(),
        has_vector_network: false,
        stack_mode: None,
        stack_spacing: None,
        stack_counter_spacing: None,
        stack_horizontal_padding: None,
        stack_vertical_padding: None,
        stack_padding_right: None,
        stack_padding_bottom: None,
        stack_primary_align_items: None,
        stack_counter_align_items: None,
        stack_counter_align_content: None,
        stack_wrap: None,
    };
    let mut has_guid = false;

    read_object(cursor, schema, def, limits, 0, |field, cursor| {
        match field.name.as_str() {
            "guid" => {
                node.guid = read_guid(cursor, schema, field, limits, 1)?;
                has_guid = true;
            }
            "parentIndex" => {
                let (parent, position) = read_parent_index(cursor, schema, field, limits, 1)?;
                node.parent = parent;
                node.position = position;
            }
            "type" => {
                let (raw, name) = read_enum(cursor, schema, field, "node type")?;
                node.kind_raw = Some(raw);
                node.kind = name;
            }
            "name" => {
                node.name = read_string_field(cursor, field, limits.max_name_bytes, "node name")?;
            }
            "phase" => {
                let (_, name) = read_enum(cursor, schema, field, "node phase")?;
                node.removed = name == "REMOVED";
                node.phase = Some(name);
            }
            "visible" => {
                node.visible = Some(read_bool_field(cursor, field, "node visible flag")?);
            }
            "size" => {
                node.size = Some(read_size(cursor, schema, field, limits, 1)?);
            }
            "transform" => {
                node.transform = Some(read_transform(cursor, schema, field, limits, 1)?);
            }
            "internalOnly" => {
                node.internal_only = read_bool_field(cursor, field, "node internalOnly flag")?;
            }
            "opacity" => {
                node.opacity = Some(read_f32_field(cursor, field, "node opacity")?);
            }
            "fillPaints" => {
                node.fills = read_paints(cursor, schema, field, limits, 1)?;
            }
            "strokePaints" => {
                node.strokes = read_paints(cursor, schema, field, limits, 1)?;
            }
            "strokeWeight" => {
                node.stroke_weight = Some(read_f32_field(cursor, field, "stroke weight")?);
            }
            "strokeAlign" => {
                node.stroke_align = Some(read_enum(cursor, schema, field, "stroke align")?.1);
            }
            "cornerRadius" => {
                node.corner_radius = Some(read_f32_field(cursor, field, "corner radius")?);
            }
            "rectangleTopLeftCornerRadius" => {
                node.corner_radii.get_or_insert([0.0; 4])[0] =
                    read_f32_field(cursor, field, "top-left corner radius")?;
            }
            "rectangleTopRightCornerRadius" => {
                node.corner_radii.get_or_insert([0.0; 4])[1] =
                    read_f32_field(cursor, field, "top-right corner radius")?;
            }
            "rectangleBottomRightCornerRadius" => {
                node.corner_radii.get_or_insert([0.0; 4])[2] =
                    read_f32_field(cursor, field, "bottom-right corner radius")?;
            }
            "rectangleBottomLeftCornerRadius" => {
                node.corner_radii.get_or_insert([0.0; 4])[3] =
                    read_f32_field(cursor, field, "bottom-left corner radius")?;
            }
            "resizeToFit" => {
                node.resize_to_fit = Some(read_bool_field(cursor, field, "resizeToFit flag")?);
            }
            "fillGeometry" => {
                node.fill_geometry = read_vector_geometry(cursor, schema, field, limits, 1)?;
            }
            "strokeGeometry" => {
                node.stroke_geometry = read_vector_geometry(cursor, schema, field, limits, 1)?;
            }
            "vectorData" => {
                node.has_vector_network = true;
                skip_field(cursor, schema, field, limits, 1)?;
            }
            "effects" => {
                node.effects = read_effects(cursor, schema, field, limits, 1)?;
            }
            "stackMode" => {
                node.stack_mode = Some(read_enum(cursor, schema, field, "stack mode")?.1);
            }
            "stackSpacing" => {
                node.stack_spacing = Some(read_f32_field(cursor, field, "stack spacing")?);
            }
            "stackCounterSpacing" => {
                node.stack_counter_spacing =
                    Some(read_f32_field(cursor, field, "stack counter spacing")?);
            }
            "stackHorizontalPadding" => {
                node.stack_horizontal_padding =
                    Some(read_f32_field(cursor, field, "stack horizontal padding")?);
            }
            "stackVerticalPadding" => {
                node.stack_vertical_padding =
                    Some(read_f32_field(cursor, field, "stack vertical padding")?);
            }
            "stackPaddingRight" => {
                node.stack_padding_right =
                    Some(read_f32_field(cursor, field, "stack right padding")?);
            }
            "stackPaddingBottom" => {
                node.stack_padding_bottom =
                    Some(read_f32_field(cursor, field, "stack bottom padding")?);
            }
            "stackPrimaryAlignItems" => {
                node.stack_primary_align_items =
                    Some(read_enum(cursor, schema, field, "stack primary alignment")?.1);
            }
            "stackCounterAlignItems" => {
                node.stack_counter_align_items =
                    Some(read_enum(cursor, schema, field, "stack counter alignment")?.1);
            }
            "stackCounterAlignContent" => {
                node.stack_counter_align_content =
                    Some(read_enum(cursor, schema, field, "stack counter content alignment")?.1);
            }
            "stackWrap" => {
                node.stack_wrap = Some(read_enum(cursor, schema, field, "stack wrap")?.1);
            }
            "textData" => {
                let text = node.text.get_or_insert_with(TextData::default);
                read_text_data(cursor, schema, field, limits, text, 1)?;
            }
            "fontName" => {
                let (family, style) = read_font_name(cursor, schema, field, limits, 1)?;
                let text = node.text.get_or_insert_with(TextData::default);
                text.font_family = family;
                text.font_style = style;
            }
            "fontSize" => {
                let size = read_f32_field(cursor, field, "font size")?;
                node.text.get_or_insert_with(TextData::default).font_size = Some(size);
            }
            "textAlignHorizontal" => {
                let align = read_enum(cursor, schema, field, "text alignment")?.1;
                node.text
                    .get_or_insert_with(TextData::default)
                    .align_horizontal = Some(align);
            }
            "textAlignVertical" => {
                let align = read_enum(cursor, schema, field, "vertical alignment")?.1;
                node.text
                    .get_or_insert_with(TextData::default)
                    .align_vertical = Some(align);
            }
            "lineHeight" => {
                let (value, units) =
                    read_number_unit(cursor, schema, field, limits, 1, "line height")?;
                let text = node.text.get_or_insert_with(TextData::default);
                text.line_height_value = Some(value);
                text.line_height_units = Some(units);
            }
            "letterSpacing" => {
                let (value, units) =
                    read_number_unit(cursor, schema, field, limits, 1, "letter spacing")?;
                let text = node.text.get_or_insert_with(TextData::default);
                text.letter_spacing_value = Some(value);
                text.letter_spacing_units = Some(units);
            }
            "textAutoResize" => {
                let resize = read_enum(cursor, schema, field, "text auto resize")?.1;
                node.text.get_or_insert_with(TextData::default).auto_resize = Some(resize);
            }
            _ => skip_field(cursor, schema, field, limits, 1)?,
        }
        Ok(())
    })?;

    if !has_guid {
        return Err(FigError::Decode(format!(
            "node of type {} has no guid",
            def.name
        )));
    }
    Ok(node)
}

fn read_vector_geometry(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Vec<VectorGeometry>> {
    if !field.is_array {
        return Err(FigError::Decode("geometry field is not an array".into()));
    }
    let count = read_count(cursor, limits.max_array_items, "geometry count")?;
    let def = schema.def(field.type_id as usize).ok_or_else(|| {
        FigError::Decode(format!("geometry type {} does not exist", field.type_id))
    })?;
    let mut geometries = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        let mut commands_blob = None;
        let mut winding_rule = None;
        read_object(cursor, schema, def, limits, depth, |field, cursor| {
            match field.name.as_str() {
                "commandsBlob" => {
                    commands_blob = Some(read_u32_field(cursor, field, "geometry commandsBlob")?);
                }
                "windingRule" => {
                    winding_rule = Some(read_enum(cursor, schema, field, "winding rule")?.1);
                }
                _ => skip_field(cursor, schema, field, limits, depth + 1)?,
            }
            Ok(())
        })?;
        geometries.push(VectorGeometry {
            commands_blob: commands_blob
                .ok_or_else(|| FigError::Decode("geometry has no commandsBlob".into()))?,
            winding_rule: winding_rule.unwrap_or_else(|| "NONZERO".to_owned()),
        });
    }
    Ok(geometries)
}

fn read_paints(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Vec<Paint>> {
    if !field.is_array {
        return Err(FigError::Decode("paint field is not an array".into()));
    }
    let count = read_count(cursor, limits.max_array_items, "paint count")?;
    let def = schema
        .def(field.type_id as usize)
        .ok_or_else(|| FigError::Decode(format!("paint type {} does not exist", field.type_id)))?;
    let mut paints = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        paints.push(read_paint(cursor, schema, def, limits, depth)?);
    }
    Ok(paints)
}

fn read_paint(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    def: &Def,
    limits: &Limits,
    depth: usize,
) -> Result<Paint> {
    let mut paint = Paint::default();
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "type" => {
                paint.kind = read_enum(cursor, schema, field, "paint type")?.1;
                if let Some(gradient) = paint.gradient.as_mut() {
                    gradient.kind = paint.kind.clone();
                }
            }
            "color" => {
                paint.color = Some(read_color(cursor, schema, field, limits, depth + 1)?);
            }
            "opacity" => {
                paint.opacity = read_f32_field(cursor, field, "paint opacity")?;
            }
            "visible" => {
                paint.visible = read_bool_field(cursor, field, "paint visible flag")?;
            }
            "stops" => {
                let gradient = paint.gradient.get_or_insert_with(|| Gradient {
                    kind: paint.kind.clone(),
                    stops: Vec::new(),
                    transform: None,
                });
                gradient.stops = read_gradient_stops(cursor, schema, field, limits, depth + 1)?;
            }
            "transform" => {
                let transform = read_transform(cursor, schema, field, limits, depth + 1)?;
                paint
                    .gradient
                    .get_or_insert_with(|| Gradient {
                        kind: paint.kind.clone(),
                        stops: Vec::new(),
                        transform: None,
                    })
                    .transform = Some(transform);
            }
            "image" => {
                paint.image_hash = read_image_hash(cursor, schema, field, limits, depth + 1)?;
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok(paint)
}

fn read_gradient_stops(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Vec<GradientStop>> {
    if !field.is_array {
        return Err(FigError::Decode("paint stops is not an array".into()));
    }
    let count = read_count(cursor, limits.max_array_items, "gradient stop count")?;
    let def = schema.def(field.type_id as usize).ok_or_else(|| {
        FigError::Decode(format!(
            "gradient stop type {} does not exist",
            field.type_id
        ))
    })?;
    let mut stops = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        let mut color = None;
        let mut position = None;
        read_object(cursor, schema, def, limits, depth, |field, cursor| {
            match field.name.as_str() {
                "color" => {
                    color = Some(read_color(cursor, schema, field, limits, depth + 1)?);
                }
                "position" => {
                    position = Some(read_f32_field(cursor, field, "gradient stop position")?);
                }
                _ => skip_field(cursor, schema, field, limits, depth + 1)?,
            }
            Ok(())
        })?;
        stops.push(GradientStop {
            color: color.ok_or_else(|| FigError::Decode("gradient stop has no color".into()))?,
            position: position
                .ok_or_else(|| FigError::Decode("gradient stop has no position".into()))?,
        });
    }
    Ok(stops)
}

fn read_image_hash(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Option<String>> {
    let def = object_def(schema, field, "paint image")?;
    let mut name = None;
    let mut hash = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "name" => {
                name = Some(read_string_field(
                    cursor,
                    field,
                    limits.max_name_bytes,
                    "image hash name",
                )?);
            }
            "hash" => {
                if !field.is_array || field.type_id != TYPE_BYTE {
                    return Err(FigError::Decode("image hash is not a byte array".into()));
                }
                let len = read_count(cursor, 64, "image hash length")?;
                let bytes = cursor
                    .read_bytes(len)
                    .map_err(|error| decode_error("image hash", error))?;
                hash = Some(bytes.iter().map(|byte| format!("{byte:02x}")).collect());
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok(name.or(hash))
}

fn read_effects(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Vec<Effect>> {
    if !field.is_array {
        return Err(FigError::Decode("effects is not an array".into()));
    }
    let count = read_count(cursor, limits.max_array_items, "effect count")?;
    let def = schema
        .def(field.type_id as usize)
        .ok_or_else(|| FigError::Decode(format!("effect type {} does not exist", field.type_id)))?;
    let mut effects = Vec::with_capacity(count.min(16));
    for _ in 0..count {
        let mut effect = Effect {
            kind: "UNKNOWN".to_owned(),
            color: None,
            offset_x: 0.0,
            offset_y: 0.0,
            radius: 0.0,
            spread: 0.0,
            visible: true,
        };
        read_object(cursor, schema, def, limits, depth, |field, cursor| {
            match field.name.as_str() {
                "type" => {
                    effect.kind = read_enum(cursor, schema, field, "effect type")?.1;
                }
                "color" => {
                    effect.color = Some(read_color(cursor, schema, field, limits, depth + 1)?);
                }
                "offset" => {
                    let def = object_def(schema, field, "effect offset")?;
                    read_object(cursor, schema, def, limits, depth + 1, |field, cursor| {
                        match field.name.as_str() {
                            "x" => {
                                effect.offset_x = read_f32_field(cursor, field, "effect offset x")?;
                            }
                            "y" => {
                                effect.offset_y = read_f32_field(cursor, field, "effect offset y")?;
                            }
                            _ => skip_field(cursor, schema, field, limits, depth + 2)?,
                        }
                        Ok(())
                    })?;
                }
                "radius" => {
                    effect.radius = read_f32_field(cursor, field, "effect radius")?;
                }
                "spread" => {
                    effect.spread = read_f32_field(cursor, field, "effect spread")?;
                }
                "visible" => {
                    effect.visible = read_bool_field(cursor, field, "effect visible")?;
                }
                _ => skip_field(cursor, schema, field, limits, depth + 1)?,
            }
            Ok(())
        })?;
        effects.push(effect);
    }
    Ok(effects)
}

fn read_color(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Color> {
    let def = object_def(schema, field, "paint color")?;
    let mut r = None;
    let mut g = None;
    let mut b = None;
    let mut a = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "r" => r = Some(read_f32_field(cursor, field, "color.r")?),
            "g" => g = Some(read_f32_field(cursor, field, "color.g")?),
            "b" => b = Some(read_f32_field(cursor, field, "color.b")?),
            "a" => a = Some(read_f32_field(cursor, field, "color.a")?),
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok(Color {
        r: r.ok_or_else(|| FigError::Decode("paint color has no r".into()))?,
        g: g.ok_or_else(|| FigError::Decode("paint color has no g".into()))?,
        b: b.ok_or_else(|| FigError::Decode("paint color has no b".into()))?,
        a: a.ok_or_else(|| FigError::Decode("paint color has no a".into()))?,
    })
}

fn read_text_data(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    text: &mut TextData,
    depth: usize,
) -> Result<()> {
    let def = object_def(schema, field, "textData")?;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "characters" => {
                let value =
                    read_string_field(cursor, field, limits.max_text_bytes, "text characters")?;
                text.characters = Some(value);
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })
}

fn read_number_unit(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
    what: &str,
) -> Result<(f32, String)> {
    let def = object_def(schema, field, what)?;
    let mut value = None;
    let mut units = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "value" => value = Some(read_f32_field(cursor, field, what)?),
            "units" => units = Some(read_enum(cursor, schema, field, what)?.1),
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok((
        value.ok_or_else(|| FigError::Decode(format!("{what} has no value")))?,
        units.ok_or_else(|| FigError::Decode(format!("{what} has no units")))?,
    ))
}

fn read_font_name(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<(Option<String>, Option<String>)> {
    let def = object_def(schema, field, "fontName")?;
    let mut family = None;
    let mut style = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "family" => {
                family = Some(read_string_field(
                    cursor,
                    field,
                    limits.max_name_bytes,
                    "font family",
                )?);
            }
            "style" => {
                style = Some(read_string_field(
                    cursor,
                    field,
                    limits.max_name_bytes,
                    "font style",
                )?);
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok((family, style))
}

fn decode_blobs(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
) -> Result<Vec<Blob>> {
    let count = read_count(cursor, limits.max_blobs, "blob count")?;
    let def = schema.def(field.type_id as usize).ok_or_else(|| {
        FigError::Decode(format!(
            "blob element type {} does not exist",
            field.type_id
        ))
    })?;
    let mut blobs = Vec::with_capacity(count.min(1_024));
    for _ in 0..count {
        let mut blob = Blob { offset: 0, len: 0 };
        let mut found = false;
        read_object(cursor, schema, def, limits, 0, |field, cursor| {
            if field.name == "bytes" {
                if !field.is_array || field.type_id != TYPE_BYTE {
                    return Err(FigError::Decode("Blob.bytes is not a byte array".into()));
                }
                let len = read_count(cursor, limits.max_blob_bytes, "blob byte count")?;
                blob.offset = cursor.position();
                cursor
                    .skip(len)
                    .map_err(|error| decode_error("blob bytes", error))?;
                blob.len = len;
                found = true;
            } else {
                skip_field(cursor, schema, field, limits, 1)?;
            }
            Ok(())
        })?;
        if !found {
            return Err(FigError::Decode("blob has no bytes field".into()));
        }
        blobs.push(blob);
    }
    Ok(blobs)
}

fn read_size(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Size> {
    let def = object_def(schema, field, "size")?;
    let mut width = None;
    let mut height = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "x" => width = Some(read_f32_field(cursor, field, "size.x")?),
            "y" => height = Some(read_f32_field(cursor, field, "size.y")?),
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok(Size {
        width: width.ok_or_else(|| FigError::Decode("size has no x".into()))?,
        height: height.ok_or_else(|| FigError::Decode("size has no y".into()))?,
    })
}

fn read_transform(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Matrix> {
    let def = object_def(schema, field, "transform")?;
    let mut values = [0.0f32; 6];
    let mut seen = [false; 6];
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        let slot = match field.name.as_str() {
            "m00" => Some(0),
            "m01" => Some(1),
            "m02" => Some(2),
            "m10" => Some(3),
            "m11" => Some(4),
            "m12" => Some(5),
            _ => None,
        };
        if let Some(index) = slot {
            values[index] = read_f32_field(cursor, field, &format!("transform.{}", field.name))?;
            seen[index] = true;
        } else {
            skip_field(cursor, schema, field, limits, depth + 1)?;
        }
        Ok(())
    })?;
    if seen.iter().any(|value| !value) {
        return Err(FigError::Decode(
            "transform is missing one or more matrix components".into(),
        ));
    }
    Ok(Matrix {
        m00: values[0],
        m01: values[1],
        m02: values[2],
        m10: values[3],
        m11: values[4],
        m12: values[5],
    })
}

fn read_guid(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<Guid> {
    let def = object_def(schema, field, "guid")?;
    let mut session_id = None;
    let mut local_id = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "sessionID" => {
                session_id = Some(read_u32_field(cursor, field, "guid.sessionID")?);
            }
            "localID" => {
                local_id = Some(read_u32_field(cursor, field, "guid.localID")?);
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok(Guid::new(
        session_id.ok_or_else(|| FigError::Decode("guid has no sessionID".into()))?,
        local_id.ok_or_else(|| FigError::Decode("guid has no localID".into()))?,
    ))
}

fn read_parent_index(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<(Option<Guid>, Option<String>)> {
    let def = object_def(schema, field, "parentIndex")?;
    let mut parent = None;
    let mut position = None;
    read_object(cursor, schema, def, limits, depth, |field, cursor| {
        match field.name.as_str() {
            "guid" => {
                let guid = read_guid(cursor, schema, field, limits, depth + 1)?;
                if !guid.is_zero() {
                    parent = Some(guid);
                }
            }
            "position" => {
                position = Some(read_string_field(
                    cursor,
                    field,
                    limits.max_name_bytes,
                    "parent index",
                )?);
            }
            _ => skip_field(cursor, schema, field, limits, depth + 1)?,
        }
        Ok(())
    })?;
    Ok((parent, position))
}

fn read_object<F>(
    cursor: &mut Cursor<'_>,
    _schema: &Schema,
    def: &Def,
    limits: &Limits,
    depth: usize,
    mut on_field: F,
) -> Result<()>
where
    F: FnMut(&Field, &mut Cursor<'_>) -> Result<()>,
{
    if depth > limits.max_depth {
        return Err(FigError::limit(
            "Kiwi nesting depth",
            depth,
            limits.max_depth,
        ));
    }
    match def.kind {
        DefKind::Struct => {
            for field in &def.fields {
                on_field(field, cursor)?;
            }
        }
        DefKind::Message => loop {
            let field_id = read_u32(cursor, "message field id")?;
            if field_id == 0 {
                break;
            }
            let field = def.field_by_value(field_id).ok_or_else(|| {
                FigError::Decode(format!("{} contains unknown field id {field_id}", def.name))
            })?;
            on_field(field, cursor)?;
        },
        DefKind::Enum => {
            return Err(FigError::Decode(format!(
                "{} is an enum, not an object",
                def.name
            )))
        }
    }
    Ok(())
}

fn skip_field(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    limits: &Limits,
    depth: usize,
) -> Result<()> {
    if field.is_array {
        let count = read_count(cursor, limits.max_array_items, "array item count")?;
        if field.type_id == TYPE_BYTE {
            if count > limits.max_blob_bytes {
                return Err(FigError::limit(
                    "byte array size",
                    count,
                    limits.max_blob_bytes,
                ));
            }
            cursor
                .skip(count)
                .map_err(|error| decode_error("byte array", error))?;
        } else {
            for _ in 0..count {
                skip_value(cursor, schema, field.type_id, limits, depth)?;
            }
        }
        Ok(())
    } else {
        skip_value(cursor, schema, field.type_id, limits, depth)
    }
}

fn skip_value(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    type_id: i32,
    limits: &Limits,
    depth: usize,
) -> Result<()> {
    if depth > limits.max_depth {
        return Err(FigError::limit(
            "Kiwi nesting depth",
            depth,
            limits.max_depth,
        ));
    }
    match type_id {
        TYPE_BOOL => {
            cursor
                .read_bool()
                .map_err(|error| decode_error("bool", error))?;
        }
        TYPE_BYTE => {
            cursor
                .read_u8()
                .map_err(|error| decode_error("byte", error))?;
        }
        TYPE_INT => {
            cursor
                .read_var_int()
                .map_err(|error| decode_error("int", error))?;
        }
        TYPE_UINT => {
            cursor
                .read_var_uint()
                .map_err(|error| decode_error("uint", error))?;
        }
        TYPE_FLOAT => {
            cursor
                .read_float()
                .map_err(|error| decode_error("float", error))?;
        }
        TYPE_STRING => {
            cursor
                .read_string(limits.max_name_bytes)
                .map_err(|error| decode_error("string", error))?;
        }
        TYPE_INT64 => {
            cursor
                .read_var_int64()
                .map_err(|error| decode_error("int64", error))?;
        }
        TYPE_UINT64 => {
            cursor
                .read_var_uint64()
                .map_err(|error| decode_error("uint64", error))?;
        }
        _ => {
            let def = schema
                .def(type_id as usize)
                .ok_or_else(|| FigError::Decode(format!("unknown Kiwi type {type_id}")))?;
            match def.kind {
                DefKind::Enum => {
                    cursor
                        .read_var_uint()
                        .map_err(|error| decode_error("enum", error))?;
                }
                DefKind::Struct => {
                    for field in &def.fields {
                        skip_field(cursor, schema, field, limits, depth + 1)?;
                    }
                }
                DefKind::Message => loop {
                    let field_id = read_u32(cursor, "nested message field id")?;
                    if field_id == 0 {
                        break;
                    }
                    let field = def.field_by_value(field_id).ok_or_else(|| {
                        FigError::Decode(format!(
                            "{} contains unknown field id {field_id}",
                            def.name
                        ))
                    })?;
                    skip_field(cursor, schema, field, limits, depth + 1)?;
                },
            }
        }
    }
    Ok(())
}

fn object_def<'a>(schema: &'a Schema, field: &Field, what: &str) -> Result<&'a Def> {
    if field.is_array {
        return Err(FigError::Decode(format!("{what} is unexpectedly an array")));
    }
    let def = schema
        .def(field.type_id as usize)
        .ok_or_else(|| FigError::Decode(format!("{what} type {} does not exist", field.type_id)))?;
    if !matches!(def.kind, DefKind::Struct | DefKind::Message) {
        return Err(FigError::Decode(format!(
            "{what} type {} is not a struct or message",
            def.name
        )));
    }
    Ok(def)
}

fn read_enum(
    cursor: &mut Cursor<'_>,
    schema: &Schema,
    field: &Field,
    what: &str,
) -> Result<(u32, String)> {
    if field.is_array {
        return Err(FigError::Decode(format!("{what} is unexpectedly an array")));
    }
    let def = schema
        .def(field.type_id as usize)
        .ok_or_else(|| FigError::Decode(format!("{what} type {} does not exist", field.type_id)))?;
    if def.kind != DefKind::Enum {
        return Err(FigError::Decode(format!(
            "{what} type {} is not an enum",
            def.name
        )));
    }
    let raw = read_u32(cursor, what)?;
    let name = def
        .field_by_value(raw)
        .map(|field| field.name.clone())
        .unwrap_or_else(|| format!("UNKNOWN({raw})"));
    Ok((raw, name))
}

fn read_u32_field(cursor: &mut Cursor<'_>, field: &Field, what: &str) -> Result<u32> {
    if field.is_array {
        return Err(FigError::Decode(format!("{what} is unexpectedly an array")));
    }
    match field.type_id {
        TYPE_UINT => read_u32(cursor, what),
        TYPE_INT => {
            let value = cursor
                .read_var_int()
                .map_err(|error| decode_error(what, error))?;
            u32::try_from(value).map_err(|_| FigError::Decode(format!("{what} is negative")))
        }
        TYPE_BYTE => Ok(cursor
            .read_u8()
            .map_err(|error| decode_error(what, error))? as u32),
        TYPE_UINT64 => {
            let value = cursor
                .read_var_uint64()
                .map_err(|error| decode_error(what, error))?;
            u32::try_from(value)
                .map_err(|_| FigError::Decode(format!("{what} does not fit in u32")))
        }
        TYPE_INT64 => {
            let value = cursor
                .read_var_int64()
                .map_err(|error| decode_error(what, error))?;
            u32::try_from(value)
                .map_err(|_| FigError::Decode(format!("{what} does not fit in u32")))
        }
        other => Err(FigError::Decode(format!(
            "{what} has non-integer type {other}"
        ))),
    }
}

fn read_bool_field(cursor: &mut Cursor<'_>, field: &Field, what: &str) -> Result<bool> {
    if field.is_array || field.type_id != TYPE_BOOL {
        return Err(FigError::Decode(format!("{what} is not a bool")));
    }
    cursor
        .read_bool()
        .map_err(|error| decode_error(what, error))
}

fn read_f32_field(cursor: &mut Cursor<'_>, field: &Field, what: &str) -> Result<f32> {
    if field.is_array || field.type_id != TYPE_FLOAT {
        return Err(FigError::Decode(format!("{what} is not a float")));
    }
    cursor
        .read_float()
        .map_err(|error| decode_error(what, error))
}

fn read_string_field(
    cursor: &mut Cursor<'_>,
    field: &Field,
    max: usize,
    what: &str,
) -> Result<String> {
    if field.is_array || field.type_id != TYPE_STRING {
        return Err(FigError::Decode(format!("{what} is not a string")));
    }
    cursor
        .read_string(max)
        .map(|value| value.to_owned())
        .map_err(|error| decode_error(what, error))
}

fn read_count(cursor: &mut Cursor<'_>, limit: usize, what: &str) -> Result<usize> {
    let count = cursor
        .read_var_uint()
        .map_err(|error| decode_error(what, error))? as usize;
    if count > limit {
        return Err(FigError::limit(what, count, limit));
    }
    Ok(count)
}

fn read_u32(cursor: &mut Cursor<'_>, what: &str) -> Result<u32> {
    cursor
        .read_var_uint()
        .map_err(|error| decode_error(what, error))
}

fn decode_error(context: &str, error: CursorError) -> FigError {
    FigError::Decode(format!("{context}: {error}"))
}
