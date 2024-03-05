
# NuTeX (.vtx)

NuTeX is a language for typesetting (mostly scientific) text-based content. It is the successor to my earlier project, [Goma](https://github.com/adrian-kriegel/goma). While Goma features a complete pipeline from source to PDF files, NuTeX doesn't. 

The goal of NuTex is to be able to write content like scientific papers or blog articles [like this one](https://adriankriegel.com/blog/you-dont-need-matrix-inversion) with much greater extensibility and much simpler syntax than LaTeX. 

## Compiler

NuTeX is simply a parser that consumes `.vtx` files and emits HTML. The output from this operation can then be passed to a renderer.

## Example

NuTeX combines elements from `HTML/XML`, `Markdown`, and `LaTeX`.  One of the key features of the language is semantics-dependent syntax. Most languages require you to escape parts of the text that should not be interpreted as syntax. NuTeX solves this by allowing you to define how the contents of your components are parsed. 


```HTML
# Example

## Text and Equations
This is an example file with example equations: $e=mc^2$

## Simple Component
You can define components like this:

<Component MyComponent>
    # ${heading}
    ${children}
</Component>

And use them like this:

<MyComponent heading="My Example Component">
    You can put anything in here.
</MyComponent>

## Component with Syntax-Breaking Content

To render things such as code or equations, you can define the parsing behavior within your components:

<Component MyCodeBlock content="raw">
    ${children}
</Component>

<MyCodeBlock>
    VTX syntax is entirely ignored in here. 
    You can <Open> up tags without closing them.
    You can also use special characters, such as #, $, and <>.
</MyCodeBlock>

```

The output format produced by the compiler: 
```HTML
<h1>Example</h1>
<h2>Text and Equations</h2>
This is an example file with example equations: 
<Eq>e=mc^2</Eq>
<h2>Simple Component</h2>
You can define components like this:
And use them like this:
<h1>My Example Component</h1>
You can put anything in here.
<h2>Component with Syntax-Breaking Content</h2>
To render things such as code or equations, you can define the parsing behavior within your components:
VTX syntax is entirely ignored in here. 
You can &lt;Open&gt; up tags without closing them.
You can also use special characters, such as #, $, and &lt;&gt;.
```
