# Tokon Tech Log

Registro de tech para **MARVEL Tōkon: Fighting Souls**. Cubre los 20 personajes del roster, con el equipo propio fijado arriba en la barra lateral.

Escrito en Rust y compilado a WebAssembly con [Yew](https://yew.rs) + [Trunk](https://trunkrs.dev). Se publica como sitio estático en GitHub Pages.

---

## Publicarlo (lo único que hace falta hacer)

El repositorio trae un workflow que compila y publica solo. Tú no necesitas tener Rust instalado para esto.

1. Crea un repositorio en GitHub y sube todos estos archivos a la raíz.
2. Ve a **Settings → Pages**.
3. En **Source**, elige **GitHub Actions**.
   *(Ojo: no "Deploy from a branch". Con este proyecto hay que compilar, y eso lo hace Actions.)*
4. Haz cualquier push a `main` (o entra en la pestaña **Actions** y lanza "Tests y despliegue" a mano).
5. En 2-4 minutos tendrás la app en `https://TU-USUARIO.github.io/TU-REPO/`.

No hay que editar ninguna ruta: el workflow saca el nombre del repositorio solo.

Cada push a `main` ejecuta primero los tests. Si fallan, no se publica nada.

---

## Trabajar en local (opcional)

Solo si quieres tocar el código y verlo en caliente.

```bash
# 1. Rust (desde https://rustup.rs)
rustup target add wasm32-unknown-unknown

# 2. Trunk
cargo install trunk

# 3. Levantar la app en http://127.0.0.1:8080 con recarga automática
trunk serve
```

Para lanzar los tests **no hace falta nada de WebAssembly**:

```bash
cargo test
```

Esto funciona porque toda la lógica que se puede testear (roster, parser de notación, CSV) vive en la librería, y las dependencias del navegador solo se compilan para `wasm32`.

---

## Versión de escritorio (.exe para Windows)

Además de la web, el proyecto se puede empaquetar como aplicación de escritorio con [Tauri](https://tauri.app). Es la misma interfaz dentro de una ventana propia, pensada para tenerla al lado del juego.

**Para conseguir el .exe sin instalar nada:** ve a la pestaña **Actions**, elige el workflow *Compilar el .exe de Windows*, dale a **Run workflow**, y cuando termine descarga el artefacto `TokonTechLog-windows` que aparece abajo del todo en esa ejecución. Dentro está `TokonTechLog.exe`.

Es portable: no se instala, se abre con doble clic y se puede dejar donde quieras. Necesita WebView2, que Windows 11 ya trae de serie.

La ventana arranca **siempre encima del resto**, en 480x780 y redimensionable. Para cambiarlo, edita `src-tauri/tauri.conf.json`:

```json
"alwaysOnTop": true,     // ponlo en false si te molesta
"width": 480,
"height": 780
```

Ojo: una ventana "siempre encima" solo se ve sobre el juego si este corre en **ventana sin bordes**, no en pantalla completa exclusiva.

Para compilarlo en tu propia máquina hace falta [el toolchain de Tauri](https://tauri.app/start/prerequisites/) y luego:

```bash
trunk build --release
cargo build --release --manifest-path src-tauri/Cargo.toml
```

El ejecutable sale en `src-tauri/target/release/`.

---

## Dónde viven los datos

Todo se guarda en el `localStorage` del navegador con el que abras la app. No hay servidor ni cuenta:

- Mismo PC y mismo navegador → tus datos siguen ahí solos.
- Desde el móvil u otro navegador → esa copia empieza vacía. **No se sincroniza entre dispositivos.**
- Si borras el historial/caché de ese navegador, se van los datos.

Usa **Exportar** de vez en cuando como copia de seguridad, y para llevar la tech de un dispositivo a otro.

Si el navegador rechaza el guardado (modo incógnito, disco lleno), la cabecera avisa con "cambios sin guardar" en vez de fallar en silencio.

---

## Notación

Escribes la notación en texto normal y se pinta sola, con los botones en color.

### Botones y direcciones

| Escribes | Significa |
|---|---|
| `L` `M` `H` `U` `A` `QS` `QA` | los botones, cada uno con su color (mayúsculas o minúsculas, da igual) |
| `2H` | ↓ y luego H |
| `236M` | quarter circle y M |
| `66` | dash adelante |
| `HH` · `LLL` | el mismo botón repetido |
| `M+H` | los dos a la vez |
| `236L/M` | una u otra |

### Aguantar y soltar

| Escribes | Significa |
|---|---|
| `[6]` | aguantar adelante |
| `[H]` | aguantar H |
| `[6H]` | aguantar dirección y botón a la vez |
| `]H[` | soltar H |
| `]4[` | soltar atrás |

Aguantar sale con subrayado verde discontinuo; soltar, con puntos grises.

### Modificadores

| Escribes | Significa |
|---|---|
| `j.` o `j` | en el aire (`j.M`, `JM`, `j.[6]HH`) |
| `g.` | en stance |
| `jc` | jump cancel (`jc9`) |
| `dl` | delay |
| `IAD` | instant air dash |
| `(JC)` | cualquier anotación entre paréntesis |

Se pintan pequeños y en verde, a propósito distintos de los botones: no son teclas.

### Enlaces entre golpes

`>` y `<` se conservan tal cual y se pintan `›` y `‹`. No se unifican, por si los usas con significados distintos. Coma, espacio y guion también separan golpes.

### Qué pasa con lo que no entiende

Se muestra tal cual, en gris. Nunca se descarta ni se corrige.

Una tirada de letras solo se convierte en botones si **toda** encaja: por eso `land` sigue siendo la palabra "land" y no L + A + "nd", y `jump` no se lee como salto + U + M.

---

## Puente con la hoja de cálculo

El botón **Plantilla** descarga un CSV con las columnas que la app entiende (también está `plantilla-tech.csv` en el repositorio):

| Personaje | Categoria | Titulo | Notacion | Rival | Nota | EnlaceX |
|---|---|---|---|---|---|---|
| Magneto | BnB | Combo básico tras anti-air | L, L, M, H, 2H | | Funciona bien en esquina | https://x.com/ejemplo |

- **Personaje** y **Titulo** son obligatorios; el resto es opcional.
- **Personaje** y **Rival** deben coincidir con el nombre tal cual sale en la app (`Doctor Doom`, `Ms. Marvel`, `Spider-Man`). Las mayúsculas dan igual.
- **Categoria** acepta `BnB`, `Matchup` o `Nota`. Lo que no reconoce se guarda como `Nota`.
- **Rival** solo se usa cuando la categoría es `Matchup`.
- El orden de las columnas da igual.

Para traer lo que ya tienes: crea una pestaña con esas columnas, pega ahí las filas de cada personaje, descárgala como CSV (Archivo → Descargar → Valores separados por comas) e **Importar**. Si alguna fila falla (personaje mal escrito, falta el título), la app te dice cuál y sigue con las demás — no se cancela la importación entera.

**Exportar** hace el camino inverso, en el mismo formato.

---

## Hashtags

Los dos chips bajo el nombre del personaje (`#TOKON_MAGNETO` y `#MARVELTōkon`) abren la búsqueda de X en una pestaña nueva, ordenada por lo más reciente. Pensado para tenerlo al lado de X mientras buscas tech.

---

## Estructura

```
src/
├── lib.rs                    lógica pura, sin navegador (aquí van los tests)
│   ├── model.rs              roster, categorías, entradas, equipo
│   ├── notation.rs           parser de notación → botones y direcciones
│   └── csv.rs                importar/exportar contra la hoja de cálculo
├── main.rs                   arranque
├── app.rs                    componente raíz: estado, barra lateral, filtros
├── platform.rs               localStorage, descargas, fechas
└── components/               formulario, tarjeta, editor de equipo, iconos
```

La separación no es decorativa: `lib.rs` y sus tres módulos no dependen de nada del navegador, así que `cargo test` funciona en cualquier máquina sin instalar el toolchain de WebAssembly.

---

## Licencia

MIT.
