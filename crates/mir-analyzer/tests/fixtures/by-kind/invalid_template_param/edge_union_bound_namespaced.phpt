===description===
Union bound in namespaced file — actual satisfying one arm should pass
===file===
<?php
namespace App;

interface Countable {}
class MyList implements Countable {}

/**
 * @template T of string|Countable
 * @param T $val
 */
function f($val): void {
    $val;
}

f('hello');      // satisfies string arm
f(new MyList()); // satisfies Countable arm
===expect===
