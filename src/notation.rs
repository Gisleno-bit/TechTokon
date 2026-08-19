//! Convierte la notacion escrita a mano en piezas que la UI puede pintar.
//!
//! El parser recorre cada golpe de izquierda a derecha y va emitiendo trozos:
//! etiquetas ("g.", "j.", "jc", "IAD"), direcciones, direcciones mantenidas
//! ("[6]"), botones y texto suelto. Esa forma libre es lo que permite tragar
//! cosas como `g.236LLL`, `j.[6]HH` o `5AA` sin tener que preverlas una a una.
//!
//! Regla de fondo: lo que escribe el usuario manda. Aqui no se corrige nada;
//! lo que no se reconoce se muestra tal cual en gris, nunca se descarta.

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

/// Prefijos que no son botones: modifican al golpe que viene detras.
/// Ordenados de mas largo a mas corto, que es como se prueban.
const PREFIJOS: &[&str] = &["iad", "jc", "dl", "ja", "j", "g"];

pub fn button_style(code: &str) -> Option<&'static ButtonStyle> {
    BUTTONS.iter().find(|b| b.code == code)
}

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

/// Cada trozo reconocible dentro de un golpe.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    /// Modificador o anotacion: "g.", "j.", "jc", "IAD", "dl", "(JC)".
    /// Se pinta pequeño y en gris: no es una tecla.
    Label(String),
    /// Direcciones ya convertidas a flechas: "236" -> "↓↘→".
    Directions(String),
    /// Lo que va entre corchetes se mantiene pulsado. Dentro puede ir una
    /// direccion ("[6]"), un boton ("[H]") o ambos ("[6H]").
    Hold(Vec<Piece>),
    /// Corchetes al reves: se suelta lo que se venia aguantando. "]H[".
    Release(Vec<Piece>),
    /// Un boton, con su color.
    Button(&'static ButtonStyle),
    /// Lo que no se ha reconocido. Se muestra sin tocar.
    Text(String),
}

/// Un golpe: la secuencia de piezas que lo componen.
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub pieces: Vec<Piece>,
}

impl Piece {
    /// Hay un boton aqui dentro, contando lo que va entre corchetes.
    pub fn tiene_boton(&self) -> bool {
        match self {
            Piece::Button(_) => true,
            Piece::Hold(dentro) | Piece::Release(dentro) => {
                dentro.iter().any(Piece::tiene_boton)
            }
            _ => false,
        }
    }

    /// Recorre esta pieza y las que lleve dentro.
    pub fn recorrer<'a>(&'a self, salida: &mut Vec<&'a Piece>) {
        salida.push(self);
        if let Piece::Hold(dentro) | Piece::Release(dentro) = self {
            for p in dentro {
                p.recorrer(salida);
            }
        }
    }
}

impl Atom {
    pub fn tiene_boton(&self) -> bool {
        self.pieces.iter().any(Piece::tiene_boton)
    }
}

/// Golpes unidos por "+" (pulsados a la vez).
#[derive(Debug, Clone, PartialEq)]
pub struct Alternative {
    pub atoms: Vec<Atom>,
}

/// Alternativas separadas por "/" (vale una u otra).
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// El conector que venia justo antes: '>' o '<'. Se conserva tal cual en
    /// vez de unificarlos, porque el usuario los usa a proposito distintos.
    /// None cuando solo habia un espacio.
    pub link: Option<char>,
    pub alternatives: Vec<Alternative>,
}

impl Group {
    pub fn tiene_boton(&self) -> bool {
        self.alternatives
            .iter()
            .any(|alt| alt.atoms.iter().any(|a| a.tiene_boton()))
    }
}

/// ¿Toda la tirada de letras son botones seguidos? "HH" -> H,H; "LLL" -> L,L,L.
/// Devuelve None si algo no encaja, para no destrozar palabras como "land".
fn descomponer_botones(run: &str) -> Option<Vec<&'static ButtonStyle>> {
    let mayus = run.to_ascii_uppercase();
    let mut salida = Vec::new();
    let mut i = 0;
    while i < mayus.len() {
        // Primero codigos de dos letras (QS, QA), luego de una.
        let encontrado = [2usize, 1].iter().find_map(|&largo| {
            mayus
                .get(i..i + largo)
                .and_then(button_style)
                .map(|st| (st, largo))
        });
        match encontrado {
            Some((st, largo)) => {
                salida.push(st);
                i += largo;
            }
            None => return None,
        }
    }
    (!salida.is_empty()).then_some(salida)
}

/// Interpreta una tirada de letras. `punto` indica si venia seguida de ".".
fn piezas_de_letras(run: &str, punto: bool, salida: &mut Vec<Piece>) {
    // "g.", "j." y cualquier otro prefijo con punto: etiqueta.
    if punto && button_style(&run.to_ascii_uppercase()).is_none() {
        salida.push(Piece::Label(format!("{run}.")));
        return;
    }

    // Botones seguidos: "M", "HH", "LLL", "QS".
    if let Some(botones) = descomponer_botones(run) {
        salida.extend(botones.into_iter().map(Piece::Button));
        return;
    }

    // Prefijo conocido y detras botones: "JM" -> j + M, "jc" -> jc.
    let minus = run.to_ascii_lowercase();
    for prefijo in PREFIJOS {
        let Some(resto) = minus.strip_prefix(prefijo) else {
            continue;
        };
        if resto.is_empty() {
            salida.push(Piece::Label(run.to_string()));
            return;
        }
        if let Some(botones) = descomponer_botones(&run[prefijo.len()..]) {
            salida.push(Piece::Label(run[..prefijo.len()].to_string()));
            salida.extend(botones.into_iter().map(Piece::Button));
            return;
        }
    }

    // Una palabra normal: "land", "held", "Freedom".
    salida.push(Piece::Text(run.to_string()));
}

/// Trocea un golpe suelto (ya sin separadores) en sus piezas.
fn parse_atom(sub: &str) -> Atom {
    let mut pieces = parse_pieces(sub);
    if pieces.is_empty() && !sub.is_empty() {
        pieces.push(Piece::Text(sub.to_string()));
    }
    Atom { pieces }
}

/// El escaner. Se llama a si mismo para el interior de los corchetes, que
/// siempre es mas corto, asi que la recursion termina sola.
fn parse_pieces(sub: &str) -> Vec<Piece> {
    let chars: Vec<char> = sub.chars().collect();
    let mut pieces = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Anotacion entre parentesis: "(JC)", "(okizeme)".
        if c == '(' {
            if let Some(rel) = chars[i..].iter().position(|&x| x == ')') {
                pieces.push(Piece::Label(chars[i..=i + rel].iter().collect()));
                i += rel + 1;
                continue;
            }
        }

        // Mantener pulsado: "[6]", "[H]", "[6H]". Lo de dentro se interpreta
        // con las mismas reglas que fuera.
        if c == '[' {
            if let Some(rel) = chars[i..].iter().position(|&x| x == ']') {
                let dentro: String = chars[i + 1..i + rel].iter().collect();
                let interior = parse_pieces(&dentro);
                if interior.is_empty() {
                    // Corchetes vacios: se muestran tal cual.
                    pieces.push(Piece::Text(chars[i..=i + rel].iter().collect()));
                } else {
                    pieces.push(Piece::Hold(interior));
                }
                i += rel + 1;
                continue;
            }
        }

        // Soltar: "]H[", corchetes al reves. Solo cuenta si hay un "[" que
        // lo cierre; un "]" suelto se trata como simbolo cualquiera.
        if c == ']' {
            if let Some(rel) = chars[i..].iter().position(|&x| x == '[') {
                let dentro: String = chars[i + 1..i + rel].iter().collect();
                let interior = parse_pieces(&dentro);
                if interior.is_empty() {
                    pieces.push(Piece::Text(chars[i..=i + rel].iter().collect()));
                } else {
                    pieces.push(Piece::Release(interior));
                }
                i += rel + 1;
                continue;
            }
        }

        if c.is_ascii_digit() {
            let inicio = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let run: String = chars[inicio..i].iter().collect();
            match run.chars().map(direction_arrow).collect::<Option<String>>() {
                Some(flechas) => pieces.push(Piece::Directions(flechas)),
                // Lleva un 0, que no es una direccion valida.
                None => pieces.push(Piece::Text(run)),
            }
            continue;
        }

        if c.is_ascii_alphabetic() {
            let inicio = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let run: String = chars[inicio..i].iter().collect();
            let punto = chars.get(i) == Some(&'.');
            piezas_de_letras(&run, punto, &mut pieces);
            if punto {
                i += 1;
            }
            continue;
        }

        // Punto suelto: se ignora, ya lo usan los prefijos.
        if c == '.' {
            i += 1;
            continue;
        }

        // Cualquier otro simbolo se acumula y se muestra tal cual.
        //
        // El primer caracter se consume SIEMPRE, antes de mirar nada. Es lo
        // que evita que un "(" o un "[" sin cerrar dejen el bucle parado en
        // el sitio: eso colgaba el hilo de la interfaz y la app entera se
        // quedaba muerta, sin ningun error, sin responder a ningun boton.
        let inicio = i;
        i += 1;
        while i < chars.len()
            && !chars[i].is_ascii_alphanumeric()
            && !matches!(chars[i], '(' | '[' | '.')
        {
            i += 1;
        }
        let run: String = chars[inicio..i].iter().collect();
        pieces.push(Piece::Text(run));
    }

    pieces
}

/// Trocea la notacion completa en grupos pintables.
pub fn parse(raw: &str) -> Vec<Group> {
    // La flecha larga es lo mismo que ">". Comas y guiones solo separan.
    let limpio = raw.replace('→', ">").replace([',', '-'], " ");
    // Los conectores pasan a ser tokens propios, aunque vengan pegados.
    let espaciado = limpio.replace('>', " > ").replace('<', " < ");

    let mut grupos = Vec::new();
    let mut link: Option<char> = None;

    for token in espaciado.split_whitespace() {
        match token {
            ">" => {
                link = Some('>');
                continue;
            }
            "<" => {
                link = Some('<');
                continue;
            }
            _ => {}
        }
        grupos.push(Group {
            link: link.take(),
            alternatives: token
                .split('/')
                .map(|alt| Alternative {
                    atoms: alt.split('+').map(parse_atom).collect(),
                })
                .collect(),
        });
    }

    grupos
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todas las piezas, incluidas las que van dentro de corchetes.
    fn piezas(raw: &str) -> Vec<Piece> {
        let grupos = parse(raw);
        let mut planas = Vec::new();
        for g in &grupos {
            for alt in &g.alternatives {
                for at in &alt.atoms {
                    for p in &at.pieces {
                        p.recorrer(&mut planas);
                    }
                }
            }
        }
        planas.into_iter().cloned().collect()
    }

    /// Solo las piezas del primer nivel, sin entrar en los corchetes.
    fn piezas_planas(raw: &str) -> Vec<Piece> {
        parse(raw)
            .iter()
            .flat_map(|g| g.alternatives.iter())
            .flat_map(|a| a.atoms.iter())
            .flat_map(|at| at.pieces.iter())
            .cloned()
            .collect()
    }

    fn botones(raw: &str) -> Vec<String> {
        piezas(raw)
            .iter()
            .filter_map(|p| match p {
                Piece::Button(b) => Some(b.code.to_string()),
                _ => None,
            })
            .collect()
    }

    fn etiquetas(raw: &str) -> Vec<String> {
        piezas(raw)
            .iter()
            .filter_map(|p| match p {
                Piece::Label(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    fn textos(raw: &str) -> Vec<String> {
        piezas(raw)
            .iter()
            .filter_map(|p| match p {
                Piece::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    // ---- lo basico de siempre ----

    #[test]
    fn cadena_basica() {
        assert_eq!(botones("L, L, M, H, 2H"), ["L", "L", "M", "H", "H"]);
        assert_eq!(parse("L, L, M, H, 2H").len(), 5);
    }

    #[test]
    fn quarter_circle() {
        let p = piezas("236M");
        assert_eq!(p[0], Piece::Directions("↓↘→".into()));
        assert_eq!(botones("236M"), ["M"]);
    }

    #[test]
    fn simultaneos_y_alternativas() {
        assert_eq!(botones("M+H"), ["M", "H"]);
        assert_eq!(botones("236L/M"), ["L", "M"]);
        assert_eq!(parse("236L/M")[0].alternatives.len(), 2);
    }

    #[test]
    fn minusculas_valen() {
        assert_eq!(botones("l m h"), ["L", "M", "H"]);
        assert_eq!(botones("236qs"), ["QS"]);
    }

    #[test]
    fn separadores_variados() {
        for raw in ["L, M, H", "L > M > H", "L → M → H", "L-M-H", "L   M    H"] {
            assert_eq!(botones(raw), ["L", "M", "H"], "fallo con {raw}");
        }
    }

    #[test]
    fn notacion_vacia() {
        assert!(parse("").is_empty());
        assert!(parse("   ").is_empty());
        assert!(parse(" , , ").is_empty());
    }

    // ---- prefijos ----

    #[test]
    fn salto_con_y_sin_punto() {
        for raw in ["JM", "jM", "j.M", "J.M", "jm"] {
            assert_eq!(botones(raw), ["M"], "fallo con {raw}");
            assert_eq!(etiquetas(raw).len(), 1, "{raw} deberia llevar etiqueta de salto");
            assert!(textos(raw).is_empty(), "{raw} no deberia dejar texto suelto");
        }
    }

    #[test]
    fn stance_de_goblin() {
        let p = piezas("g.236LLL");
        assert_eq!(p[0], Piece::Label("g.".into()));
        assert_eq!(p[1], Piece::Directions("↓↘→".into()));
        assert_eq!(botones("g.236LLL"), ["L", "L", "L"]);
    }

    #[test]
    fn una_palabra_que_empieza_por_prefijo_no_lo_es() {
        for palabra in ["jump", "juggle", "land", "held", "grab", "Freedom", "Charge"] {
            assert_eq!(
                textos(palabra),
                vec![palabra.to_string()],
                "{palabra} deberia quedarse como texto"
            );
            assert!(botones(palabra).is_empty(), "{palabra} no lleva botones");
        }
    }

    #[test]
    fn palabras_clave_sueltas() {
        for kw in ["dl", "IAD", "jc"] {
            assert_eq!(etiquetas(kw), vec![kw.to_string()]);
        }
    }

    // ---- direcciones ----

    #[test]
    fn direccion_mantenida_entre_corchetes() {
        let p = piezas_planas("j.[6]HH");
        assert_eq!(p[0], Piece::Label("j.".into()));
        assert_eq!(p[1], Piece::Hold(vec![Piece::Directions("→".into())]));
        assert_eq!(botones("j.[6]HH"), ["H", "H"]);
    }

    #[test]
    fn boton_mantenido() {
        // "[H]" es aguantar el boton, no una etiqueta de texto.
        let p = piezas_planas("[H]");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0], Piece::Hold(vec![Piece::Button(button_style("H").unwrap())]));
        assert!(p[0].tiene_boton(), "el boton de dentro debe contar");
    }

    #[test]
    fn direccion_y_boton_mantenidos_a_la_vez() {
        let p = piezas_planas("[6H]");
        let Piece::Hold(dentro) = &p[0] else {
            panic!("deberia ser un Hold, es {:?}", p[0]);
        };
        assert_eq!(dentro[0], Piece::Directions("→".into()));
        assert_eq!(dentro[1], Piece::Button(button_style("H").unwrap()));
    }

    #[test]
    fn mantener_combinado_con_prefijos() {
        // "j.[6]H" y "g.[6]M" son los casos reales del usuario.
        assert_eq!(botones("j.[6]H"), ["H"]);
        assert_eq!(botones("g.[6]M"), ["M"]);
        // Y aguantar el boton dentro del stance.
        let p = piezas_planas("g.[H]");
        assert_eq!(p[0], Piece::Label("g.".into()));
        assert!(matches!(p[1], Piece::Hold(_)));
    }

    #[test]
    fn soltar_con_corchetes_al_reves() {
        let p = piezas_planas("]H[");
        assert_eq!(p.len(), 1);
        assert_eq!(
            p[0],
            Piece::Release(vec![Piece::Button(button_style("H").unwrap())])
        );
    }

    #[test]
    fn aguantar_y_soltar_en_la_misma_cadena() {
        // El caso tipico: se carga y mas adelante se suelta.
        let g = parse("5[H] > 2M > ]H[");
        assert_eq!(g.len(), 3);
        let primera = &g[0].alternatives[0].atoms[0].pieces;
        assert!(matches!(primera[1], Piece::Hold(_)), "{primera:?}");
        let ultima = &g[2].alternatives[0].atoms[0].pieces;
        assert!(matches!(ultima[0], Piece::Release(_)), "{ultima:?}");
    }

    #[test]
    fn soltar_una_direccion() {
        let p = piezas_planas("]4[");
        assert_eq!(p[0], Piece::Release(vec![Piece::Directions("←".into())]));
    }

    #[test]
    fn soltar_cuenta_sus_botones() {
        assert_eq!(botones("]H[ > 236L"), ["H", "L"]);
        assert!(piezas_planas("]H[")[0].tiene_boton());
    }

    #[test]
    fn un_corchete_de_cierre_suelto_no_cuelga() {
        for raw in ["]", "]H", "5H]", "]]", "][", "]["] {
            let g = parse(raw);
            assert!(!g.is_empty(), "{raw} se ha quedado vacio");
        }
    }

    #[test]
    fn corchetes_vacios_no_rompen() {
        let p = piezas_planas("[]");
        assert_eq!(p, vec![Piece::Text("[]".into())]);
    }

    #[test]
    fn dash_y_direcciones_repetidas() {
        assert_eq!(piezas("66")[0], Piece::Directions("→→".into()));
        assert_eq!(piezas("g.22L")[1], Piece::Directions("↓↓".into()));
    }

    #[test]
    fn jump_cancel_con_direccion() {
        let p = piezas("jc9");
        assert_eq!(p[0], Piece::Label("jc".into()));
        assert_eq!(p[1], Piece::Directions("↗".into()));
    }

    #[test]
    fn el_cero_no_es_direccion() {
        assert_eq!(textos("10L"), vec!["10".to_string()]);
    }

    // ---- botones repetidos ----

    #[test]
    fn botones_repetidos() {
        assert_eq!(botones("5AA"), ["A", "A"]);
        assert_eq!(botones("j.MMM"), ["M", "M", "M"]);
        assert_eq!(botones("g.236LL"), ["L", "L"]);
    }

    // ---- conectores ----

    #[test]
    fn se_distingue_el_conector() {
        let g = parse("j.U < g.236LL > g.623A");
        assert_eq!(g[0].link, None, "el primero no lleva conector");
        assert_eq!(g[1].link, Some('<'));
        assert_eq!(g[2].link, Some('>'));
    }

    #[test]
    fn conector_pegado_sin_espacios() {
        let g = parse("5M>2M>5H");
        assert_eq!(g.len(), 3);
        assert_eq!(g[1].link, Some('>'));
    }

    #[test]
    fn las_palabras_sueltas_no_llevan_conector() {
        let g = parse("dl j.U");
        assert_eq!(g.len(), 2);
        assert!(g[0].link.is_none());
        assert!(g[1].link.is_none(), "un espacio no es un conector");
    }

    // ---- los nueve combos reales de Green Goblin ----

    const COMBOS: &[&str] = &[
        "j.5M > 5M > 2M > 5H > jc9 j.[6]HH > j.U > g.236LLL > g.9U > j.M > j.[6]HH > dl j.U < g.236LL > g.623A > g.623A",
        "5A > 5M > 2M > 5H > jc9 j.[6]HH > j.U > g.236LLL > g.9U > j.M > j.[6]HH > dl j.U < g.236LL > g.623A > g.623A",
        "5M > 2M > 5H > jc9 j.[6]HH > j.U > g.236LL > g.22L > IAD > j.6H > j9 j.[6]HH > dl j.U < g.236LL > g.623A < g.623A",
        "2M > 5H > jc9 j.[6]HH > j.U > g.236LL > g.22L > IAD > j.6H > 5H > jc9 j.[6]HH > dl j.U < g.236LL > g.623A < g.623A",
        "5M > 2H > 214M > 66 > 5M > 5H > jc9 j.[6]HH > dl j.U < g.[6]M < g.6U > j.MM > j.U < g.236LL > g.623A < g.623A",
        "j.M > 2H > 214M > 66 > 5M > 5H > jc9 j.[6]HH > dl j.U < g.236LLL < g.6U > j.MM > j.U < g.236LL > g.623A < g.623A",
        "5AA > 2H > 214M > 66 > 5M > jc9 j.[6]HH > dl j.U < g.[6]M < g.6U > j.MM > j.U < g.236LL > g.623A < g.623A",
        "2H > j.MMM > j.H > j.M > jc9 j.MM > 5H > jc9 > j.[6]HH > j.U > g.236LL > g.623L > g.623L",
        "g.5M > g.6U > j.5MM > j.U > g.236LL > g.22L > IAD > j.6H > 5H > jc9 j.M > j.[6]HH > dl j.U > g.236LL > g.623L > g.623L",
    ];

    #[test]
    fn los_combos_reales_no_dejan_texto_sin_reconocer() {
        for (n, combo) in COMBOS.iter().enumerate() {
            let sueltos = textos(combo);
            assert!(
                sueltos.is_empty(),
                "combo {}: ha quedado texto sin interpretar: {:?}",
                n + 1,
                sueltos
            );
        }
    }

    #[test]
    fn los_combos_reales_tienen_botones_en_casi_todos_los_grupos() {
        for (n, combo) in COMBOS.iter().enumerate() {
            let grupos = parse(combo);
            assert!(grupos.len() > 10, "combo {}: se ha troceado mal", n + 1);
            // Un grupo puede no llevar boton: "jc9" es un salto cancelado,
            // "66" un dash, "dl" un retardo. Lo que no vale es que quede
            // texto sin reconocer, que es señal de que el parser se ha
            // rendido con algo.
            for g in &grupos {
                let piezas: Vec<&Piece> = g
                    .alternatives
                    .iter()
                    .flat_map(|a| a.atoms.iter())
                    .flat_map(|at| at.pieces.iter())
                    .collect();
                assert!(!piezas.is_empty(), "combo {}: grupo vacio", n + 1);
                assert!(
                    !piezas.iter().any(|p| matches!(p, Piece::Text(_))),
                    "combo {}: grupo con texto sin reconocer: {:?}",
                    n + 1,
                    piezas
                );
            }
        }
    }

    #[test]
    fn combo_uno_pieza_a_pieza() {
        let g = parse(COMBOS[0]);
        // "j.5M"
        assert_eq!(
            g[0].alternatives[0].atoms[0].pieces,
            vec![
                Piece::Label("j.".into()),
                Piece::Directions("•".into()),
                Piece::Button(button_style("M").unwrap()),
            ]
        );
        // "jc9" y "j.[6]HH" van seguidos separados por espacio, sin conector
        let jc = g.iter().find(|x| {
            x.alternatives[0].atoms[0].pieces.first() == Some(&Piece::Label("jc".into()))
        });
        assert!(jc.is_some(), "no se ha encontrado el jc9");
        // El total de botones del combo
        assert_eq!(botones(COMBOS[0]).len(), 19);
    }

    #[test]
    fn simbolos_sin_cerrar_no_cuelgan_el_parser() {
        // Cada uno de estos colgaba la app entera. Si vuelve a pasar, este
        // test se queda bloqueado y la compilacion falla por tiempo.
        for raw in [
            "j.[6HH",
            "(JC j.M",
            "5H > [ > 2M",
            "5H > ( > 2M",
            "j.[",
            "(",
            "[",
            "))",
            "]]",
            "[[6]]",
            "((JC))",
        ] {
            let grupos = parse(raw);
            assert!(!grupos.is_empty(), "{raw} no deberia quedarse vacio");
        }
    }

    #[test]
    fn nada_se_pierde_aunque_falte_el_cierre() {
        // El texto sigue estando, aunque no se entienda.
        let p = piezas("j.[6HH");
        assert!(p.iter().any(|x| matches!(x, Piece::Label(t) if t == "j.")));
        assert!(p.iter().any(|x| matches!(x, Piece::Text(t) if t == "[")));
        assert_eq!(botones("j.[6HH"), ["H", "H"]);
    }

    #[test]
    fn una_barra_doble_no_deja_etiquetas_vacias() {
        let p = piezas("L//M");
        assert!(
            !p.iter().any(|x| matches!(x, Piece::Text(t) if t.is_empty())),
            "no deberia haber texto vacio: {p:?}"
        );
    }

    #[test]
    fn cada_boton_de_la_leyenda_tiene_color() {
        for (code, _) in LEGEND {
            assert!(button_style(code).is_some(), "falta color para {code}");
        }
    }
}
