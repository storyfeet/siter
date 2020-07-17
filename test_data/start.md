>---toml
a=[3, 4, 5]
g=5
d=2
>---
This should be unprocessed
>---go
{{- range $x := .a }}
    <p>{{$x}} is a number</p>
{{- end}}
>---md
Hello
-----

>---exec cat file.txt

>---front

