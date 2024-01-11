# gempost

gempost is a minimal static site generator for publishing a blog (gemlog) on
the [Gemini protocol](https://geminiprotocol.net/).

You store metadata about each gemlog post in a sidecar YAML file, and gempost
generates a gemtext index page and an Atom feed.

You can use a [Tera](https://keats.github.io/tera/) template to customize the
format of the index page. You can also use a template to customize the format
of the gemlog posts themselves, such as to add a copyright footer or a
navigation header to each post. There are examples of both under
[examples/templates/](./examples/templates/).

The metadata in the sidecar YAML file allows you to generate an Atom feed with
rich metadata, but most of this metadata is optional and not necessary to
generate a working feed.

## Examples

- See [examples/gempost.yaml](./examples/gempost.yaml) For an example config
  file you can use to get started.
- See [examples/metadata/](./examples/metadata/) For examples of the sidecar
  YAML metadata files.
- See [examples/templates/](./examples/templates/) For examples of index page
  templates and post page templates.

## Templates

The index page template has access to a `feed` variable which is a Feed object.

The post page template has access to an `entry` variable which is an Entry
object.

### Author object

- `name` (string)
- `email` (string, optional)
- `uri` (string, optional)

### Entry object

- `id` (string)
- `uri` (string)
- `title` (string)
- `body` (string)
- `summary` (string, optional)
- `updated` (string, RFC 3339 format)
- `published` (string, RFC 3339 format, optional)
- `author` (Author object, optional)
- `rights` (string, optional)
- `lang` (string, optional)

### Feed object

- `title` (string)
- `updated` (string, RFC 3339 format)
- `subtitle` (string, optional)
- `rights` (string, optional)
- `author` (Author object, optional)
- `entries` (array of Entry objects)
