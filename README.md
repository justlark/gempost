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

## Getting started

To install gempost, you must first [install
Rust](https://www.rust-lang.org/tools/install). Then, you can install gempost
with Cargo.

```shell
cargo install --git https://github.com/justlark/gempost.git gempost
```

You can initialize a new gempost project like this:

```shell
gempost init ./capsule/
```

This will create a directory `./capsule/` and populate it with:

- An example `gempost.yaml` config file to get you started. You'll need to edit
  this to set your capsule's title and URI.
- Some basic templates you can use as-is or customize (an index page template
  and a post page template).
- An "hello world" example post for your gemlog, with its accompanying sidecar
  metadata file.
- A static `index.gmi` for your capsule root.

Edit the `gempost.yaml`, and then you're ready to build your capsule!

```shell
cd ./capsule
gempost build
```

Your capsule will be generated in the `./public/` directory. You'll need a
Gemini server like [Agate](https://github.com/mbrubeck/agate) to actually serve
your capsule over the Gemini protocol.

You can add new posts to your gemlog by creating a `.gmi` file in the
`./posts/` directory with an accompanying `.yaml` file with the same filename.
See [examples/metadata.yaml](./examples/metadata.yaml) for an example of all
the different values you can set in the YAML metadata file. Only some are
required.

You can add new static content to your capsule (anything that's not your
gemlog) by putting it in the `./static/` directory.

You can customize the index page and post page templates in the `./templates/`
directory from their defaults. They use the
[Tera](https://keats.github.io/tera/) text templating language, which is
similar to the popular Jinja templating language. See the
[Templates](#templates) section below for a list of all the variables that are
available inside these template.

## Examples

- See [examples/index.tera](./examples/index.tera) for an example of an index
  page template.
- See [examples/post.tera](./examples/post.tera) for an example of a post page
  template.
- See [examples/metadata.yaml](./examples/metadata.yaml) for an example of a
  sidecar gemlog post metadata file showing all the possible fields.

## Templates

The index page template has access to a `feed` variable which is a Feed object.

The post page template has access to an `entry` variable which is an Entry
object.

### Author object

- `name` (string)
- `email` (string, optional)
- `uri` (string, optional)

### Entry object

- `uri` (string)
- `title` (string)
- `body` (string)
- `summary` (string, optional)
- `updated` (string, RFC 3339 format)
- `published` (string, RFC 3339 format, optional)
- `author` (Author object, optional)
- `rights` (string, optional)
- `lang` (string, optional)
- `categories` (array of strings, optional)

### Feed object

- `title` (string)
- `updated` (string, RFC 3339 format)
- `subtitle` (string, optional)
- `rights` (string, optional)
- `author` (Author object, optional)
- `entries` (array of Entry objects)
