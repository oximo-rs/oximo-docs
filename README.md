# oximo-docs

Documentation site for [**oximo**](https://github.com/oximo-rs/oximo), a Rust algebraic
modeling library for mathematical optimization. Define optimization models declaratively in
Rust. Solver-agnostic and type-safe.

Built with [Zola](https://www.getzola.org/), a static site generator. Live at
[oximo.dev](https://oximo.dev).

## Structure

| Path          | What                                                     |
| ------------- | -------------------------------------------------------- |
| `content/`    | Markdown docs pages (`_index.md`, `quickstart.md`, ...)  |
| `templates/`  | Tera templates. `base.html` is the page shell            |
| `sass/`       | Styles (compiled to CSS by Zola)                         |
| `static/`     | Assets served as-is: JS, fonts, images, wasm             |
| `config.toml` | Site config: base URL, markdown, `[extra]` theme options |

## Develop

Requires [Zola](https://www.getzola.org/documentation/getting-started/installation/).

```sh
zola serve # dev server with live reload
zola build # output static site to public/
zola check # validate links and markup
```

Edit docs in `content/`. Edit layout/styling in `templates/` and `sass/`.

## Analytics

This site uses [Umami](https://umami.is/) for privacy-friendly, cookieless analytics. The
self-hosted tracker (`static/js/um.js`) is loaded from `<head>` in `templates/base.html`. No
personal data or cookies are collected.

## License

[MIT](LICENSE.txt)

This project is heavily modified from the [Tanuki](https://github.com/raskell-io/tanuki) Zola theme, used under the MIT license. The original theme license is retained in [LICENSE-TANUKI.txt](LICENSE-TANUKI.txt).
