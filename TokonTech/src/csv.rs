//! El puente con la hoja de calculo: leer y escribir CSV.
//!
//! Se escribe a mano en vez de tirar de una crate porque el formato es fijo y
//! asi la lib no arrastra dependencias al wasm final.

use crate::model::{
    character_by_name, character_name, normalize_url, Category, Entry, TechData,
};

pub const HEADERS: &[&str] = &[
    "Personaje",
    "Categoria",
    "Titulo",
    "Notacion",
    "Rival",
    "Nota",
    "EnlaceX",
];

/// Fila de ejemplo que acompana a la plantilla descargable.
pub fn template_rows() -> Vec<Vec<String>> {
    vec![
        HEADERS.iter().map(|s| s.to_string()).collect(),
        vec![
            "Magneto".into(),
            "BnB".into(),
            "Combo basico tras anti-air".into(),
            "L, L, M, H, 2H".into(),
            String::new(),
            "Funciona bien en esquina".into(),
            "https://x.com/ejemplo".into(),
        ],
    ]
}

/// Parser de CSV con comillas: soporta comas y saltos de linea dentro de un
/// campo, y comillas escapadas como "".
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    // Una comilla solo abre un tramo entrecomillado si esta al principio del
    // campo. En medio es texto literal, que es como se comporta Excel.
    let mut at_field_start = true;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if at_field_start => {
                    in_quotes = true;
                    at_field_start = false;
                }
                ',' => {
                    row.push(std::mem::take(&mut field));
                    at_field_start = true;
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    at_field_start = true;
                }
                '\r' => {}
                other => {
                    field.push(other);
                    at_field_start = false;
                }
            }
        }
    }

    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }

    // Las lineas en blanco del final de la hoja no cuentan como filas.
    rows.retain(|r| r.iter().any(|f| !f.trim().is_empty()));
    rows
}

fn escape_field(value: &str) -> String {
    if value.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn to_csv(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|r| r.iter().map(|f| escape_field(f)).collect::<Vec<_>>().join(","))
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// Toda la tech guardada, en filas listas para abrir en Sheets o Excel.
pub fn export_rows(data: &TechData) -> Vec<Vec<String>> {
    let mut rows = vec![HEADERS.iter().map(|s| s.to_string()).collect::<Vec<_>>()];

    // Se recorre el roster en orden para que el CSV salga siempre igual.
    for character in crate::model::CHARACTERS {
        let Some(entries) = data.get(character.id) else {
            continue;
        };
        for entry in entries {
            rows.push(vec![
                character.name.to_string(),
                entry.category.label().to_string(),
                entry.title.clone(),
                entry.notation.clone(),
                entry
                    .rival_id
                    .as_deref()
                    .map(character_name)
                    .unwrap_or("")
                    .to_string(),
                entry.note.clone(),
                entry.x_link.clone(),
            ]);
        }
    }
    rows
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ImportResult {
    pub added: usize,
    pub errors: Vec<String>,
}

fn column_index(header: &[String], name: &str) -> Option<usize> {
    header
        .iter()
        .position(|h| h.trim().to_lowercase() == name)
}

fn cell(row: &[String], index: Option<usize>) -> String {
    index
        .and_then(|i| row.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Mete las filas de un CSV en `data`. Una fila mala no aborta la importacion:
/// se apunta el problema y se sigue con las demas.
pub fn import_into(
    text: &str,
    data: &mut TechData,
    now: f64,
    next_id: &mut dyn FnMut() -> String,
) -> ImportResult {
    let rows = parse_csv(text);
    let Some(header) = rows.first() else {
        return ImportResult {
            added: 0,
            errors: vec!["El archivo esta vacio.".to_string()],
        };
    };

    let idx_personaje = column_index(header, "personaje");
    let idx_titulo = column_index(header, "titulo");
    if idx_personaje.is_none() || idx_titulo.is_none() {
        return ImportResult {
            added: 0,
            errors: vec![
                "Faltan columnas obligatorias: Personaje y Titulo. Descarga la plantilla para ver el formato."
                    .to_string(),
            ],
        };
    }

    let idx_categoria = column_index(header, "categoria");
    let idx_notacion = column_index(header, "notacion");
    let idx_rival = column_index(header, "rival");
    let idx_nota = column_index(header, "nota");
    let idx_enlace = column_index(header, "enlacex");

    let mut result = ImportResult::default();

    for (offset, row) in rows.iter().skip(1).enumerate() {
        let numero_fila = offset + 2; // +1 por la cabecera, +1 porque las hojas empiezan en 1

        let nombre = cell(row, idx_personaje);
        let Some(character) = character_by_name(&nombre) else {
            result
                .errors
                .push(format!("Fila {numero_fila}: no reconozco el personaje \"{nombre}\"."));
            continue;
        };

        let title = cell(row, idx_titulo);
        if title.is_empty() {
            result
                .errors
                .push(format!("Fila {numero_fila}: falta el titulo."));
            continue;
        }

        let category = Category::from_cell(&cell(row, idx_categoria));
        let rival_id = if category == Category::Matchup {
            character_by_name(&cell(row, idx_rival)).map(|c| c.id.to_string())
        } else {
            None
        };

        data.entry(character.id.to_string())
            .or_default()
            .push(Entry {
                id: next_id(),
                title,
                notation: cell(row, idx_notacion),
                category,
                rival_id,
                note: cell(row, idx_nota),
                x_link: normalize_url(&cell(row, idx_enlace)),
                created_at: now,
            });
        result.added += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TechData;

    fn contador() -> impl FnMut() -> String {
        let mut n = 0;
        move || {
            n += 1;
            format!("id{n}")
        }
    }

    #[test]
    fn campos_con_comas_se_respetan() {
        let csv = "Personaje,Titulo\nMagneto,\"Combo, basico\"";
        let rows = parse_csv(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1][1], "Combo, basico");
    }

    #[test]
    fn comillas_escapadas() {
        let rows = parse_csv("A,B\n\"dijo \"\"hola\"\"\",normal");
        assert_eq!(rows[1][0], "dijo \"hola\"");
        assert_eq!(rows[1][1], "normal");
    }

    #[test]
    fn comillas_sueltas_en_medio_de_un_campo_son_texto() {
        // Si alguien edita el CSV a mano y escribe comillas sin entrecomillar
        // el campo, se conservan tal cual en vez de desaparecer.
        let rows = parse_csv("A,B\nCon \"comillas\" dentro,x");
        assert_eq!(rows[1][0], "Con \"comillas\" dentro");
    }

    #[test]
    fn saltos_de_linea_dentro_de_un_campo() {
        let rows = parse_csv("A,B\n\"linea1\nlinea2\",x");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1][0], "linea1\nlinea2");
    }

    #[test]
    fn filas_vacias_se_ignoran() {
        let rows = parse_csv("A,B\n\n\nx,y\n");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn crlf_no_deja_retornos_sueltos() {
        let rows = parse_csv("A,B\r\nx,y\r\n");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn ida_y_vuelta_conserva_los_campos_dificiles() {
        let original = vec![
            vec!["Personaje".to_string(), "Nota".to_string()],
            vec!["Blade".to_string(), "Tiene, comas y \"comillas\"".to_string()],
            vec!["Storm".to_string(), "Nota\ncon salto".to_string()],
        ];
        let vuelta = parse_csv(&to_csv(&original));
        assert_eq!(vuelta, original);
    }

    #[test]
    fn importa_filas_validas_y_reporta_las_malas() {
        let csv = concat!(
            "Personaje,Categoria,Titulo,Notacion,Rival,Nota,EnlaceX\n",
            "Magneto,BnB,\"Combo esquina, largo\",L L M H 2H,,\"Nota con, coma\",x.com/m1\n",
            "Doctor Doom,Matchup,Anti-air vs Storm,2H,Storm,Cuidado con el dash,\n",
            "PersonajeQueNoExiste,Nota,Titulo raro,,,,\n",
            "Blade,Nota,,,,,\n"
        );
        let mut data = TechData::new();
        let result = import_into(csv, &mut data, 1000.0, &mut contador());

        assert_eq!(result.added, 2);
        assert_eq!(result.errors.len(), 2);
        assert!(result.errors[0].contains("PersonajeQueNoExiste"));
        assert!(result.errors[1].contains("falta el titulo"));

        let magneto = &data["magneto"][0];
        assert_eq!(magneto.title, "Combo esquina, largo");
        assert_eq!(magneto.note, "Nota con, coma");
        assert_eq!(magneto.x_link, "https://x.com/m1", "debe anadir el esquema");
        assert_eq!(magneto.category, Category::Bnb);

        let doom = &data["doom"][0];
        assert_eq!(doom.category, Category::Matchup);
        assert_eq!(doom.rival_id.as_deref(), Some("storm"));
    }

    #[test]
    fn el_rival_se_ignora_si_no_es_matchup() {
        let csv = "Personaje,Categoria,Titulo,Rival\nBlade,BnB,Combo,Storm";
        let mut data = TechData::new();
        import_into(csv, &mut data, 0.0, &mut contador());
        assert_eq!(data["blade"][0].rival_id, None);
    }

    #[test]
    fn cabecera_incompleta_no_importa_nada() {
        let csv = "Cosa,Otra\nx,y";
        let mut data = TechData::new();
        let result = import_into(csv, &mut data, 0.0, &mut contador());
        assert_eq!(result.added, 0);
        assert!(data.is_empty());
        assert!(result.errors[0].contains("Faltan columnas"));
    }

    #[test]
    fn el_orden_de_las_columnas_da_igual() {
        let csv = "Titulo,EnlaceX,Personaje\nCombo raro,x.com/z,Hulk";
        let mut data = TechData::new();
        let result = import_into(csv, &mut data, 0.0, &mut contador());
        assert_eq!(result.added, 1);
        assert_eq!(data["hulk"][0].title, "Combo raro");
        assert_eq!(data["hulk"][0].x_link, "https://x.com/z");
    }

    #[test]
    fn importar_no_borra_lo_que_ya_habia() {
        let mut data = TechData::new();
        import_into(
            "Personaje,Titulo\nHulk,Primera",
            &mut data,
            0.0,
            &mut contador(),
        );
        import_into(
            "Personaje,Titulo\nHulk,Segunda",
            &mut data,
            0.0,
            &mut contador(),
        );
        assert_eq!(data["hulk"].len(), 2);
    }

    #[test]
    fn exportar_e_importar_cierra_el_circulo() {
        // El campo con comillas viene entrecomillado, que es como lo exporta
        // cualquier hoja de calculo.
        let csv_inicial = concat!(
            "Personaje,Categoria,Titulo,Notacion,Rival,Nota,EnlaceX\n",
            "Magneto,BnB,\"Combo, raro\",236L/M,,\"Con \"\"comillas\"\"\",https://x.com/a\n",
            "Doctor Doom,Matchup,Vs Storm,2H,Storm,,\n"
        );
        let mut data = TechData::new();
        import_into(csv_inicial, &mut data, 0.0, &mut contador());

        let exportado = to_csv(&export_rows(&data));
        let mut data2 = TechData::new();
        let result = import_into(&exportado, &mut data2, 0.0, &mut contador());

        assert_eq!(result.added, 2);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(data2["magneto"][0].title, "Combo, raro");
        assert_eq!(data2["magneto"][0].note, "Con \"comillas\"");
        assert_eq!(data2["magneto"][0].notation, "236L/M");
        assert_eq!(data2["doom"][0].rival_id.as_deref(), Some("storm"));
    }

    #[test]
    fn la_plantilla_se_importa_sin_errores() {
        let csv = to_csv(&template_rows());
        let mut data = TechData::new();
        let result = import_into(&csv, &mut data, 0.0, &mut contador());
        assert_eq!(result.added, 1);
        assert!(result.errors.is_empty());
    }
}
