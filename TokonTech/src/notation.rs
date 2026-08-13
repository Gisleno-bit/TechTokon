//! Convierte la notacion escrita a mano ("L, L, M, H, 2H", "236M", "M+H")
//! en algo que la UI pueda pintar como botones de colores.
//!
//! El texto que escribe el usuario manda: aqui no se corrige nada, solo se
//! reconoce lo que se puede y el resto se muestra tal cual.

/// Colores de cada boton de Tokon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonStyle {
    pub code: &'static str,
    pub bg: &'static str,
    pub fg: &'static str,
}

pub const BUTTONS: &[ButtonStyle] = &[
    ButtonStyle { code: "L", bg: "#F2C94C", fg: "#241D00" },
    ButtonStyle { code: "M", bg: "#F2782F", fg: "#2A1200" },
    ButtonStyle { code: "H", bg: "#E85D5D", fg: "#2A0808" },
    ButtonStyle { code: "U", bg: "#8C7CF7", fg: "#170F35" },
    ButtonStyle { code: "A", bg: "#4FBDE8", fg: "#00232E" },
    ButtonStyle { code: "QS", bg: "#B5E84C", fg: "#182400" },
    ButtonStyle { code: "QA", bg: "#4FE8C0", fg: "#00251C" },
];

/// Los cinco botones que se explican en la leyenda de la app.
pub const LEGEND: &[(&str, &str)] = &[
    ("L", "Ligero"),
    ("M", "Medio"),
    ("H", "Pesado"),
    ("U", "Unica"),
    ("A", "Assemble"),
];

pub fn button_style(code: &str) -> Option<&'static ButtonStyle> {
    BUTTONS.iter().find(|b| b.code == code)
}

/// Direccion del numpad a flecha.
fn direction_arrow(c: char) -> Option<char> {
    match c {
        '1' => Some('↙'),
        '2' => Some('↓'),
        '3' => Some('↘'),
        '4' => Some('←'),
        '5' => Some('•'),
        '6' => Some('→'),
        '7' => Some('↖'),
        '8' => Some('↑'),
        '9' => Some('↗'),
        _ => None,
    }
}

/// La unidad minima: unas direcciones opcionales y un boton, o texto suelto
/// que no supimos interpretar.
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    /// Flechas ya convertidas, p.ej. "↓↘" para "23".
    pub directions: String,
    pub button: Option<&'static ButtonStyle>,
    /// Texto que no encaja en el esquema; se pinta en gris tal cual.
    pub fallback: Option<String>,
}

/// Atomos unidos por "+" (pulsados a la vez).
#[derive(Debug, Clone, PartialEq)]
pub struct Alternative {
    pub atoms: Vec<Atom>,
}

/// Alternativas separadas por "/" (vale una u otra).
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub alternatives: Vec<Alternative>,
}

/// Separadores que la gente usa entre golpes: coma, guion, flecha, mayor que.
fn normalize_separators(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            ',' | '-' | '>' | '→' => ' ',
            other => other,
        })
        .collect()
}

/// Todo lo que no encaja se muestra en gris, sin tocarlo.
fn plain(sub: &str) -> Atom {
    Atom {
        directions: String::new(),
        button: None,
        fallback: Some(sub.to_string()),
    }
}

fn parse_atom(sub: &str) -> Atom {
    if sub.is_empty() {
        return plain(sub);
    }

    let digits: String = sub.chars().take_while(|c| c.is_ascii_digit()).collect();
    let rest: String = sub.chars().skip(digits.chars().count()).collect();

    // Direcciones validas son 1-9; un 0 invalida el tramo entero.
    let arrows: Option<String> = digits.chars().map(direction_arrow).collect();
    let arrows = match arrows {
        Some(a) => a,
        None => return plain(sub),
    };

    if rest.is_empty() {
        // Solo direcciones, p.ej. "236".
        return Atom { directions: arrows, button: None, fallback: None };
    }

    if !rest.chars().all(|c| c.is_ascii_alphabetic()) {
        // Mezcla rara como "j.L" o "5[H]": se muestra entera sin interpretar.
        return plain(sub);
    }

    match button_style(&rest.to_ascii_uppercase()) {
        // Direcciones + boton conocido: el caso normal.
        Some(style) => Atom { directions: arrows, button: Some(style), fallback: None },
        // Palabra que no es un boton ("2Special"): conservamos ambas partes.
        None => Atom { directions: arrows, button: None, fallback: Some(rest) },
    }
}

/// Trocea la notacion completa en grupos pintables.
pub fn parse(raw: &str) -> Vec<Group> {
    normalize_separators(raw)
        .split_whitespace()
        .map(|token| Group {
            alternatives: token
                .split('/')
                .map(|alt| Alternative {
                    atoms: alt.split('+').map(parse_atom).collect(),
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solo_botones(raw: &str) -> Vec<String> {
        parse(raw)
            .iter()
            .flat_map(|g| g.alternatives.iter())
            .flat_map(|a| a.atoms.iter())
            .filter_map(|at| at.button.map(|b| b.code.to_string()))
            .collect()
    }

    #[test]
    fn cadena_basica_de_botones() {
        let grupos = parse("L, L, M, H, 2H");
        assert_eq!(grupos.len(), 5);
        assert_eq!(solo_botones("L, L, M, H, 2H"), ["L", "L", "M", "H", "H"]);
    }

    #[test]
    fn quarter_circle_se_convierte_en_flechas() {
        let grupos = parse("236M");
        assert_eq!(grupos.len(), 1);
        let atom = &grupos[0].alternatives[0].atoms[0];
        assert_eq!(atom.directions, "↓↘→");
        assert_eq!(atom.button.unwrap().code, "M");
        assert!(atom.fallback.is_none());
    }

    #[test]
    fn botones_simultaneos_con_mas() {
        let grupos = parse("M+H");
        assert_eq!(grupos[0].alternatives.len(), 1);
        assert_eq!(grupos[0].alternatives[0].atoms.len(), 2);
        assert_eq!(solo_botones("M+H"), ["M", "H"]);
    }

    #[test]
    fn alternativas_con_barra() {
        let grupos = parse("236L/M");
        assert_eq!(grupos.len(), 1);
        assert_eq!(grupos[0].alternatives.len(), 2);
        assert_eq!(solo_botones("236L/M"), ["L", "M"]);
    }

    #[test]
    fn minusculas_tambien_valen() {
        assert_eq!(solo_botones("l m h"), ["L", "M", "H"]);
        assert_eq!(solo_botones("236qs"), ["QS"]);
    }

    #[test]
    fn separadores_variados_dan_el_mismo_resultado() {
        let esperado = ["L", "M", "H"];
        assert_eq!(solo_botones("L, M, H"), esperado);
        assert_eq!(solo_botones("L > M > H"), esperado);
        assert_eq!(solo_botones("L → M → H"), esperado);
        assert_eq!(solo_botones("L-M-H"), esperado);
        assert_eq!(solo_botones("L   M    H"), esperado);
    }

    #[test]
    fn texto_desconocido_se_conserva_tal_cual() {
        let grupos = parse("j.QQQ");
        let atom = &grupos[0].alternatives[0].atoms[0];
        assert!(atom.button.is_none());
        assert_eq!(atom.fallback.as_deref(), Some("j.QQQ"));
    }

    #[test]
    fn direccion_sola_sin_boton() {
        let grupos = parse("236");
        let atom = &grupos[0].alternatives[0].atoms[0];
        assert_eq!(atom.directions, "↓↘→");
        assert!(atom.button.is_none());
        assert!(atom.fallback.is_none());
    }

    #[test]
    fn notacion_vacia_no_produce_grupos() {
        assert!(parse("").is_empty());
        assert!(parse("   ").is_empty());
        assert!(parse(" , , ").is_empty());
    }

    #[test]
    fn combo_largo_realista() {
        let raw = "2L, 5M, 236H, M+H, j.L";
        let grupos = parse(raw);
        assert_eq!(grupos.len(), 5);
        // No debe romperse ni perder informacion por el camino.
        assert_eq!(solo_botones(raw), ["L", "M", "H", "M", "H"]);
    }

    #[test]
    fn cada_boton_tiene_color_definido() {
        for (code, _) in LEGEND {
            assert!(button_style(code).is_some(), "falta color para {code}");
        }
    }
}
