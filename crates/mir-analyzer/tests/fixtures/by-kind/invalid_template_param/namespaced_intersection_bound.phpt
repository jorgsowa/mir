===description===
Intersection bound with namespaced types should be FQN-qualified and pass when actual satisfies both parts
===file===
<?php
namespace App;

interface Type {}
interface Named {}

class Both implements Type, Named {}

/**
 * @template T of Type&Named
 * @param T $t
 */
function f($t): void {
    echo get_class($t);
}

f(new Both());
===expect===
