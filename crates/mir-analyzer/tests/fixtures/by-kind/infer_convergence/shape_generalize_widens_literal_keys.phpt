===description===
When a shape generalizes to a plain array (here: past its size cap), the
key domain widens to `string`/`int` instead of accumulating each property's
own literal key — otherwise a run of distinctly-keyed writes produces a
swelling union like `"a"|"b"|"c"|...` rather than the `string` a generic
array's key type should actually be. Values still keep their precise
literal union, matching how this codebase treats value precision elsewhere.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $arr = [];
    $arr['a'] = 1;
    $arr['b'] = 2;
    $arr['c'] = 3;
    $arr['d'] = 4;
    $arr['e'] = 5;
    $arr['f'] = 6;
    $arr['g'] = 7;
    $arr['h'] = 8;
    $arr['i'] = 9;
    /** @mir-check $arr is array<string, 9|1|2|3|4|5|6|7|8> */
    $_ = $arr;
}
===expect===
