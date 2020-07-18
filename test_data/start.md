>---toml
a=[3, 4, 5]
gmd=["cows","sheep","fish"]
d=2
>---#
This is a comment
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

>---exec cat test_data/part.md |md

>---go|md
{{- range $k ,$x := .gmd}}
Title {{$k}}
==========

Thing with {{$x}} and stuff
{{end}}
>---exec grep cat
this cat is big
this dog is small
catthew

