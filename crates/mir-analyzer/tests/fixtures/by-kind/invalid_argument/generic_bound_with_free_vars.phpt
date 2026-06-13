===description===
subclass of a generic base with concrete type params should satisfy template bound with free type vars
===config===
suppress=InvalidCast
===file===
<?php
/**
 * @template K
 * @template V
 */
class Base {}

class Concrete extends Base {}

/**
 * @template T of Base<K, V>
 * @template K
 * @template V
 */
function g(Base $t): void {
    echo (string) $t;
}

g(new Concrete());
===expect===
