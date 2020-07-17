>---toml
a=[3, 4, 5]
gmd=["cows","sheep","fish"]
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

>---table class="pic"
* |car|Daog|poo|
  |No |Food| Soss|
- |more | od|

>---exec cat file.txt

>---go|md
{{- range $k ,$x := .gmd}}
Title {{$k}}
==========

Thing with {{$x}} and stuff
{{end}}


