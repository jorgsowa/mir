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
//       ^^^^^^^^^ UndefinedFunction: Function nope() is not defined
    $g = namespace\gone(...);
//       ^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function App\gone() is not defined
    $h = \Lib\missing(...);
//       ^^^^^^^^^^^^^^^^^ UndefinedFunction: Function Lib\missing() is not defined
    $k = known(...);
    $l = strlen(...);
    $m = \strtoupper(...);
    if (function_exists('maybe')) {
        $n = maybe(...);
        $n();
    }
    $f(); $g(); $h(); $k(); $l('a'); $m('a');
}
