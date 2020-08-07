{{export title="Hello"}}
{{@md @}}
I will set "aa" to "Hello world"
-----------
{{@let aa}}\
    Hello world\
{{/let}}

aa is now "{{$aa}}"

{{/md}}
