===description===
P3: First-class callable from an inherited method resolves through the ancestor chain.
===config===
suppress=UnusedVariable
===file===
<?php

class Base {
    public function compute(int $x): float { return (float) $x; }
}

class Child extends Base {}

$c = new Child();
$fn = $c->compute(...);
/** @mir-check $fn is Closure(int): float */
$_ = $fn;
===expect===
