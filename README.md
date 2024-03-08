
# NuTeX (.vtx)

NuTeX is a language for typesetting (mostly scientific) text-based content. It is the successor to my earlier project, [Goma](https://github.com/adrian-kriegel/goma). While Goma features a complete pipeline from source to PDF files, NuTeX doesn't. 

The goal of NuTeX is to be able to write content like scientific papers or blog articles [like this one](https://adriankriegel.com/blog/you-dont-need-matrix-inversion) with much greater extensibility and much simpler syntax than LaTeX. 

## Environments

NuTeX combines elements from `HTML/XML`, `Markdown`, and `LaTeX`. At the core of the language are environments. You can explicitly open/close environments using `XML`-like tags: 
```XML
<SomeEnvironment>Content</SomeEnvironment>
```

But since this is cumbersome, NuTeX defines aliases for some environments: 

```HTML
# Headings
## Like in Markdown

/** is equivalent to */

<h1>Headings</h1>
<h2>Like in Markdown</h2>
```

```HTML
$e=mc^2$

/** is equivalent to */

<Eq>e=mc^2</Eq>
```

## Variables

You can declare variables which will be defined within an environment and its children

```HTML 
<var MyVariable="MyValue" />
<var MyComplexVariable>
    # Put anything in here.
</var>
```
and insert them like this:

```HTML
${MyVariable}
${MyComplexVariable}
``` 

Variables are scoped within the environment they are defined in. This means you can re-define a variable within a nested environment without affecting the original value:

```HTML 
<var MyVariable="MyValue" />

<SomeEnvironment>
    <var MyVariable="MyOtherValue">
    /** Resolves to "MyOtherValue" */
    ${MyVariable}
</SomeEnvironment>

/** Resolves to "MyValue" */
${MyVariable}
```

## Components

You can define your own environments using components 

```HTML
<Component MyComponent>
    # ${heading}
    ${children}
</Component>
```
and use them like this:
```HTML
<MyComponent heading="My Example Component">
    You can put anything in here.
</MyComponent>
```

## Semantics-Dependent Syntax

One of the key features of the language is *semantics-dependent syntax*. 

Most languages require you to escape parts of the text that should not be interpreted as NuTeX syntax. NuTeX solves this by allowing you to define how the contents of custom environments are parsed. 

```HTML
<Component MyCodeBlock content="raw">
    ${children}
</Component>

<MyCodeBlock>
    VTX syntax is entirely ignored in here. 
    You can <Open> up tags without closing them.
    You can also use special characters, such as #, $, and <>.
</MyCodeBlock>
``` 

## Under the Hood

This section is intended for anyone interested in how the language is processed under the hood.

### Parsing

`.vtx` files are parsed directly into a tree of nodes. This is done by creating a mutable `Parser` on the source. The `Parser` performs tokenization while building the document tree simultaneously. This is important in order to enable semantics-dependent parsing.  

```Rust
let mut parser = Parser::new(src);

let document = parser.parse_document();
```

### Syntax Errors

`parser.parse_document()` always returns a document. Syntax errors will lead to the insertion of an error node in the document tree. This allows the parser to recover the document structure and continue parsing in most cases. Any syntax errors are emitted as error tokens.

### Document Tree
Calling `parser.parse_document()` returns a root node representing what is *almost* an abstract syntax tree, though some nuances of the syntax are not represented. These include environment aliases.

### Transformers

The `plugins` module contains a set of submodules which each expose one or more visitors. These visitors may perform manipulations on the document tree, and are responsible for implementing language features. Depending on which visitors are used and how they are arranged, different behaviors can be achieved. 