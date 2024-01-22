# gempost

gempost is a minimal static site generator for publishing a blog (gemlog) on
the [Gemini protocol](https://geminiprotocol.net/).

You store metadata about each gemlog post in a sidecar YAML file, and gempost
generates a gemtext index page and an Atom feed.

You can use a [Tera](https://keats.github.io/tera/) template to customize the
format of the index page. You can also use a template to customize the format
of the gemlog posts themselves, such as to add a copyright footer or a
navigation header to each post. See [Examples](#Examples) for examples of both.

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
  this to set your capsule's title and URL.
- Some basic templates you can use as-is or customize.
- A "hello world" example post for your gemlog, with its accompanying sidecar
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

Running `gempost init` will generate minimal index and post page templates you
can use to get started. These examples make use of more of the post metadata to
provide more rich output. You can use these verbatim, or use them as examples
to write your own.

- See [examples/index.tera](./examples/index.tera) for an example of an index
  page template.
- See [examples/post.tera](./examples/post.tera) for an example of a post page
  template.
- See [examples/metadata.yaml](./examples/metadata.yaml) for an example of a
  sidecar gemlog post metadata file showing all the possible fields.

## Templates

The index page template has access to:
- A `feed` variable which is a Feed object.

The post page template has access to:
- A `feed` variable which is a Feed object.
- An `entry` variable which is an Entry object for the current post.

All dates are in RFC 3339 format, which looks like this:

```
2006-01-02T15:04:05Z07:00
```

### Author object

- `name` *(string)* The name of the author
- `email` *(string, optional)* The author's email address
- `uri` *(string, optional)* A URI describing the author

### Entry object

- `url` *(string)* The URL of the post
- `title` *(string)* The title of the post
- `body` *(string)* The gemtext body of the post
- `updated` *(string)* When the post was last updated
- `summary` *(string, optional)* The summary of the post
- `published` *(string, optional)* When the post was originally published
- `author` *(Author object, optional)* The author of the post
- `rights` *(string, optional)* The copyright and license information for the post
- `lang` *(string, optional)* The RFC 5646 language code for the language the
  post is written in (e.g. `en`, `de`)
- `categories` *(array of strings)* The list of categories the post belongs to

### Feed object

- `capsule_url` *(string)* The URL of your capsule's homepage
- `feed_url` *(string)* The URL of the Atom feed
- `index_url` *(string)* The URL of the gemlog index page
- `title` *(string)* The title of the feed
- `updated` *(string)* When any post in the feed was last updated
- `subtitle` *(string, optional)* The subtitle of the feed
- `rights` *(string, optional)* The copyright and license information for the feed
- `author` *(Author object, optional)* The primary author of the feed
- `entries` *(array of Entry objects)* The list of posts in the feed, sorted
  reverse-chronologically by publish date or, if no publish date, last updated
  date

## Similar tools

Check out these other static site generators for gemlogs:

- [gloggery](https://github.com/kconner/gloggery)
- [gssg](https://git.sr.ht/~gsthnz/gssg)
- [kiln](https://git.sr.ht/~adnano/kiln)
