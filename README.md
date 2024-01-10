# gempost

gempost is a minimal static site generator for publishing a blog (gemlog) on
the [Gemini protocol](https://geminiprotocol.net/).

You store metadata about each gemlog post in a sidecar YAML file, and gempost
generates a gemtext index page and an Atom feed.

You can use a [Tera](https://keats.github.io/tera/) template to customize the
format of the index page. You can also use a template to customize the format
of the gemlog posts themselves, such as to add a copyright footer or a
navigation header to each post. There are examples of both under
[examples/](./examples/).

The metadata in the sidecar YAML file allows you to generate an Atom feed with
rich metadata, but most of this metadata is optional and not necessary to
generate a working feed.
