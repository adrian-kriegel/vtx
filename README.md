# DISCLAIMER 

This project is in development. I started it as my first project in my journey of learning Rust. Many things could probably be done better, and I appreciate any input. I intentionally avoided using too many dependencies in order to properly learn the language. There are crates with exceptional language-parsing APIs that one could (and should) use instead of writing everything by hand.


# NuTeX (.vtx)

NuTeX is a language for typesetting (mostly scientific) text-based content. It is the successor to my earlier project, [Goma](https://github.com/adrian-kriegel/goma). While Goma features a complete pipeline from source to PDF files, NuTeX doesn't. 

The goal of NuTex is to be able to write content like scientific papers or blog articles [like this one](https://adriankriegel.com/blog/you-dont-need-matrix-inversion) with much greater extensibility and much simpler syntax than LaTeX. 

## Compiler

NuTeX is simply a parser that consumes `.vtx` files and emits intermediate formats, which can be defined using plugins. The output from this operation can then be passed to a renderer. Compiling to an intermediate format has the advantage of allowing different render targets to be configurable.


## Example

NuTex combines elements from `HTML/XML`, `Markdown`, and `LaTeX`:

```
# Heading

## Second Level Heading

This is an example file with example equations: $e=mc^2$

Another equation: 

<Eq>
    \nu Te \mathcal{X}
</Eq>
```

The intermediate format produced by the compiler: 
```
<h1>Heading</h1>
<h2>Second Level Heading</h2>
This is an example file with example equations: <Eq>e=mc^2</Eq>

Another equation: 

<Eq block >
    \nu Te \mathcal{X}
</Eq>
```

## Roadmap

- Make the intermediate format XML/HTML compatible.
- Create a web renderer (I wrote one for Next.js but would like a framework-agnostic one).
- Create a pagination engine that would allow for producing PDF files.

## Open Questions

I haven't made my mind up yet about a couple of questions. You can find these in the issues tab.
