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
| `dev/`        | Development-only page overrides, published under `/dev/` |
| `config.toml` | Site config: base URL, markdown, `[extra]` theme options |

## Develop

Requires [Zola](https://www.getzola.org/documentation/getting-started/installation/).

```sh
zola serve # dev server with live reload
zola build # output static site to public/
zola check # validate links and markup
```

Edit docs in `content/`. Edit layout/styling in `templates/` and `sass/`.
The deployment-equivalent combined build is `bash ./build.sh`. It publishes both the stable
root and the `/dev/` channel.

`zola serve` serves only the stable content tree.

## Documentation channels

The root site is the stable documentation channel. The combined Cloudflare build also
publishes a development channel at [oximo.dev/dev](https://oximo.dev/dev/). The development
build starts from the stable content and overlays files from `dev/`, so a page only needs to
be copied into `dev/` when it diverges from the latest release.

To release a new stable version, promote the tested development files into `content/`, update
`stable_version` and the version metadata in `config.toml`, and merge that promotion through
the release branch. The build then continues to publish both stable `/` and development `/dev/`
from one artifact.

## Analytics

This site uses [Umami](https://umami.is/) for privacy-friendly, cookieless analytics. The
self-hosted tracker (`static/js/um.js`) is loaded from `<head>` in `templates/base.html`. No
personal data or cookies are collected.

## License

[MIT](LICENSE.txt)

This project is heavily modified from the [Tanuki](https://github.com/raskell-io/tanuki) Zola theme, used under the MIT license. The original theme license is retained in [LICENSE-TANUKI.txt](LICENSE-TANUKI.txt).
