//! Roster, categorias y tipos de datos que se guardan en disco.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Un personaje del roster base de Tokon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Character {
    pub id: &'static str,
    pub name: &'static str,
    /// Monograma de dos letras que se ve en la barra lateral.
    pub mono: &'static str,
}

/// Los 20 personajes del roster de lanzamiento.
pub const CHARACTERS: &[Character] = &[
    Character { id: "doom", name: "Doctor Doom", mono: "DD" },
    Character { id: "cap", name: "Captain America", mono: "CA" },
    Character { id: "magneto", name: "Magneto", mono: "MG" },
    Character { id: "blade", name: "Blade", mono: "BL" },
    Character { id: "blackpanther", name: "Black Panther", mono: "BP" },
    Character { id: "carnage", name: "Carnage", mono: "CN" },
    Character { id: "danger", name: "Danger", mono: "DG" },
    Character { id: "deadpool", name: "Deadpool", mono: "DP" },
    Character { id: "ghostrider", name: "Ghost Rider", mono: "GR" },
    Character { id: "greengoblin", name: "Green Goblin", mono: "GG" },
    Character { id: "hulk", name: "Hulk", mono: "HK" },
    Character { id: "ironman", name: "Iron Man", mono: "IM" },
    Character { id: "loki", name: "Loki", mono: "LK" },
    Character { id: "magik", name: "Magik", mono: "MK" },
    Character { id: "msmarvel", name: "Ms. Marvel", mono: "MM" },
    Character { id: "peniparker", name: "Peni Parker", mono: "PP" },
    Character { id: "spiderman", name: "Spider-Man", mono: "SM" },
    Character { id: "starlord", name: "Star-Lord", mono: "SL" },
    Character { id: "storm", name: "Storm", mono: "ST" },
    Character { id: "wolverine", name: "Wolverine", mono: "WV" },
];

/// Equipo por defecto la primera vez que se abre la app. Editable desde la UI.
pub const DEFAULT_TEAM_IDS: &[&str] = &["doom", "cap", "magneto", "blade"];
pub const DEFAULT_TEAM_NAME: &str = "Mi equipo";

pub fn character_by_id(id: &str) -> Option<&'static Character> {
    CHARACTERS.iter().find(|c| c.id == id)
}

/// Busca por nombre ignorando mayusculas y espacios sobrantes: lo que hace
/// falta para que el import de CSV perdone erratas de la hoja de calculo.
pub fn character_by_name(name: &str) -> Option<&'static Character> {
    let needle = name.trim().to_lowercase();
    CHARACTERS.iter().find(|c| c.name.to_lowercase() == needle)
}

pub fn character_name(id: &str) -> &str {
    character_by_id(id).map(|c| c.name).unwrap_or("")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Bnb,
    Matchup,
    Nota,
}

impl Category {
    pub const ALL: [Category; 3] = [Category::Bnb, Category::Matchup, Category::Nota];

    pub fn label(&self) -> &'static str {
        match self {
            Category::Bnb => "BnB",
            Category::Matchup => "Matchup",
            Category::Nota => "Nota",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Category::Bnb => "bnb",
            Category::Matchup => "matchup",
            Category::Nota => "nota",
        }
    }

    /// Lee una categoria de una celda de CSV. Lo que no reconoce cae en Nota,
    /// para no perder la fila entera por una errata.
    pub fn from_cell(raw: &str) -> Category {
        match raw.trim().to_lowercase().as_str() {
            "bnb" => Category::Bnb,
            "matchup" => Category::Matchup,
            _ => Category::Nota,
        }
    }
}

/// Una tech guardada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub notation: String,
    pub category: Category,
    #[serde(default)]
    pub rival_id: Option<String>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub x_link: String,
    /// Milisegundos desde epoch, igual que `Date.now()`.
    #[serde(default)]
    pub created_at: f64,
}

/// Lo que devuelve el formulario antes de convertirse en `Entry`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryDraft {
    pub title: String,
    pub notation: String,
    pub category: Category,
    pub rival_id: Option<String>,
    pub note: String,
    pub x_link: String,
}

/// Toda la tech, indexada por id de personaje.
pub type TechData = HashMap<String, Vec<Entry>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamConfig {
    pub name: String,
    pub ids: Vec<String>,
}

impl Default for TeamConfig {
    fn default() -> Self {
        TeamConfig {
            name: DEFAULT_TEAM_NAME.to_string(),
            ids: DEFAULT_TEAM_IDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl TeamConfig {
    /// Descarta ids que no existan en el roster (por ejemplo si un guardado
    /// viejo trae un personaje que ya no esta).
    pub fn sanitized(mut self) -> Self {
        self.ids.retain(|id| character_by_id(id).is_some());
        if self.name.trim().is_empty() {
            self.name = DEFAULT_TEAM_NAME.to_string();
        }
        self
    }

    pub fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|x| x == id)
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(pos) = self.ids.iter().position(|x| x == id) {
            self.ids.remove(pos);
        } else {
            self.ids.push(id.to_string());
        }
    }
}

/// Anade el esquema si falta, para que un enlace pegado a medias siga abriendo.
pub fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

/// El hashtag que usa la comunidad para la tech de un personaje.
pub fn hashtag_for(name: &str) -> String {
    let clean: String = name
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    format!("#TOKON_{clean}")
}

/// Enlace de busqueda en X para un hashtag, ordenado por lo mas reciente.
pub fn x_search_url(hashtag: &str) -> String {
    let encoded = hashtag.replace('#', "%23");
    format!("https://x.com/search?q={encoded}&f=live")
}

const MESES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

/// Formatea "dia mes" a partir de dia y mes ya extraidos, para poder testearlo
/// sin depender del reloj del navegador.
pub fn format_day_month(day: u32, month_index: usize) -> String {
    let mes = MESES.get(month_index).copied().unwrap_or("");
    format!("{day:02} {mes}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_tiene_20_personajes_con_ids_unicos() {
        assert_eq!(CHARACTERS.len(), 20);
        let mut ids: Vec<&str> = CHARACTERS.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 20, "hay ids de personaje repetidos");

        let mut monos: Vec<&str> = CHARACTERS.iter().map(|c| c.mono).collect();
        monos.sort_unstable();
        monos.dedup();
        assert_eq!(monos.len(), 20, "hay monogramas repetidos");
    }

    #[test]
    fn el_equipo_por_defecto_existe_en_el_roster() {
        for id in DEFAULT_TEAM_IDS {
            assert!(character_by_id(id).is_some(), "{id} no esta en el roster");
        }
    }

    #[test]
    fn busca_personaje_por_nombre_sin_importar_mayusculas() {
        assert_eq!(character_by_name("doctor doom").unwrap().id, "doom");
        assert_eq!(character_by_name("  Ms. Marvel ").unwrap().id, "msmarvel");
        assert_eq!(character_by_name("SPIDER-MAN").unwrap().id, "spiderman");
        assert!(character_by_name("Ryu").is_none());
    }

    #[test]
    fn categoria_desconocida_cae_en_nota() {
        assert_eq!(Category::from_cell("BnB"), Category::Bnb);
        assert_eq!(Category::from_cell(" matchup "), Category::Matchup);
        assert_eq!(Category::from_cell("cualquier cosa"), Category::Nota);
        assert_eq!(Category::from_cell(""), Category::Nota);
    }

    #[test]
    fn normaliza_urls_a_medias() {
        assert_eq!(normalize_url("x.com/algo"), "https://x.com/algo");
        assert_eq!(normalize_url("https://x.com/a"), "https://x.com/a");
        assert_eq!(normalize_url("HTTP://x.com/a"), "HTTP://x.com/a");
        assert_eq!(normalize_url("   "), "");
    }

    #[test]
    fn hashtags_por_personaje() {
        assert_eq!(hashtag_for("Magneto"), "#TOKON_MAGNETO");
        assert_eq!(hashtag_for("Ms. Marvel"), "#TOKON_MSMARVEL");
        assert_eq!(hashtag_for("Spider-Man"), "#TOKON_SPIDERMAN");
    }

    #[test]
    fn el_equipo_se_limpia_de_ids_invalidos() {
        let team = TeamConfig {
            name: "  ".to_string(),
            ids: vec!["magneto".into(), "personaje-fantasma".into(), "storm".into()],
        }
        .sanitized();
        assert_eq!(team.ids, vec!["magneto".to_string(), "storm".to_string()]);
        assert_eq!(team.name, DEFAULT_TEAM_NAME);
    }

    #[test]
    fn toggle_anade_y_quita_del_equipo() {
        let mut team = TeamConfig::default();
        assert!(team.contains("doom"));
        team.toggle("doom");
        assert!(!team.contains("doom"));
        team.toggle("storm");
        assert!(team.contains("storm"));
    }

    #[test]
    fn las_entradas_sobreviven_al_viaje_por_json() {
        let entry = Entry {
            id: "abc".into(),
            title: "Combo esquina".into(),
            notation: "L, L, M, H, 2H".into(),
            category: Category::Matchup,
            rival_id: Some("storm".into()),
            note: "Cuidado con el dash".into(),
            x_link: "https://x.com/a".into(),
            created_at: 1_700_000_000_000.0,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"rivalId\""), "json: {json}");
        assert!(json.contains("\"matchup\""), "json: {json}");
        let vuelta: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, vuelta);
    }
}
