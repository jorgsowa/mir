===description===
Namespaced docblock keywords are not resolved as classes.
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
