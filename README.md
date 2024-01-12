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

## Getting started

To install gempost, you must first [install
Rust](https://www.rust-lang.org/tools/install). Then, you can install gempost
with Cargo.

```shell
cargo install --git https://github.com/justlark/gempost.git gempost
```

Start by creating a directory for your new Gemini capsule.

```shell
mkdir ./capsule
```

You should also clone the gempost repo so you can access the example config
files and templates. Alternatively, you can just download or copy them from
GitHub.

```shell
git clone https://github.com/justlark/gempost ./gempost
```

Copy the example `gempost.yaml` config file into your project directory. Make
sure you edit it to specify the title of your gemlog and your capsule's URI.

```shell
cp ./gempost/examples/gempost.yaml ./capsule/
nano ./capsule/gempost.yaml
```

You'll need an index page template and a post page template. Start with these
minimal example templates; you can customize them later.

```shell
mkdir ./capsule/templates
cp ./gempost/examples/templates/index.minimal.tera ./capsule/templates/index.tera
cp ./gempost/examples/templates/post.minimal.tera ./capsule/templates/post.tera
```

Let's add your first gemlog post! In the `./posts/` directory, add a
`hello-world.gmi` and a `hello-world.yaml`. The gemtext file contains the
contents of your post, and the YAML file contains metadata, like the title of
your post and when it was last updated. These files need to have the same name
(sans file extension).

```shell
mkdir ./capsule/posts
cp ./gemini/examples/metadata/hello-world.minimal.yaml ./capsule/posts/hello-world.yaml
echo "Hello, world!" > ./capsule/posts/hello-world.gmi
```

The rest of your capsule (everything except your gemlog) goes in the
`./static/` directory. At a minimum, you'll need an `index.gmi`.
```shell
mkdir ./capsule/static
cp ./gemini/examples/index.gmi ./capsule/static/index.gmi
```

Now you're ready to build your site!

```shell
cd ./capsule
gempost build
```

Your site will be generated in the `./public/` directory. You'll need a Gemini
server like [Agate](https://github.com/mbrubeck/agate) to actually serve your
capsule over the Gemini protocol.

## Examples

- See [examples/gempost.yaml](./examples/gempost.yaml) For an example config
  file you can use to get started.
- See [examples/metadata/](./examples/metadata/) For examples of the sidecar
  YAML metadata files.
- See [examples/templates/](./examples/templates/) For examples of index page
  templates and post page templates.

The examples of metadata and template files are split into "minimal" examples
that will get you started, and "complete" examples that demonstrate more of the
features of gempost.

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

### Feed object

- `title` (string)
- `updated` (string, RFC 3339 format)
- `subtitle` (string, optional)
- `rights` (string, optional)
- `author` (Author object, optional)
- `entries` (array of Entry objects)
