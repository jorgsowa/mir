===description===
A first-class callable of an undefined function is reported like a call, as
PHP throws when it creates it; defined, builtin and `function_exists()`-guarded
names are not.
===file===
<?php
namespace App;

function known(): int { return 1; }

function a(): void {
    $f = nope(...);
    $g = namespace\gone(...);
    $h = \Lib\missing(...);
    $k = known(...);
    $l = strlen(...);
    $m = \strtoupper(...);
    if (function_exists('maybe')) {
        $n = maybe(...);
        $n();
    }
    $f(); $g(); $h(); $k(); $l('a'); $m('a');
}
===expect===
UndefinedFunction@7:9-7:18: Function nope() is not defined
UndefinedFunction@8:9-8:28: Function App\gone() is not defined
UndefinedFunction@9:9-9:26: Function Lib\missing() is not defined
