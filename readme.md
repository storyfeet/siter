Siter
======

A static website generator, where every content item is a template and so are the templates.

The advantage of making content pages also templates is that you can sneak javascript into them for little bits and pieces, or loops without having to reorganise everything to fit them.

The setup for a basic website looks something like this.

* root\_config.ito
* content
    * index.md
    * anotherpage.md
    * apples.md
* templates
    * page.html
* static
    * any static files


root\_config.ito
```none
{{export
output="public"
}}
{{@export menu}}
<a href="/">HOME</a>
<a href="/apples">apples</a>
{{/export}}

```

index.md
```none
{{export
title="My Website"
}}{{@md}}
Markdown
=======

By limiting markdown to specifig blocks we can include other kinds of content outside

{{/md}}
<script>
    function not_markdown(){
        return "This function will not be gobbled up by markdown"
    }
</script>
{{@md}}
Realisitically most content pages won't need javascript, but it's nice to know you CAN.
{{/md}}
```

templates/page.html

```html
{{export as_index=true}}
<doctype! HTML>
<html>
<head>
    <title>{{first .title "My Default Title"}}</title>
    {{#If the page has any head data include that here#}}
    {{first .head}}
</head>
<body>
    <div id="menu">{{.menu}}</div>
    <div id="content">
        {{$1}}{{# $1 is the page content #}}
    </div>
</body>
</html>

```
