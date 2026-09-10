===description===
Docblock keywords in a namespaced file are never namespace-qualified into
nonexistent classes: the `@param` keeps its keyword type (so argument checks
run against the builtin, not `App\arraylike-object`), and `@var` annotations
for the generic-only keywords `key-of`/`value-of` resolve as mixed, not as
classes.
===file===
<?php
namespace App;

/**
 * @param arraylike-object $o
 * @return int
 */
function f($o) {
    /** @var key-of $k */
    $k = 0;
    /** @var value-of $v */
    $v = "x";
    return count([$o, $k, $v]);
}

f(new \stdClass());

===expect===
