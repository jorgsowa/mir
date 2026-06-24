===description===
P3: Instance method first-class callable $obj->method(...) produces a typed Closure.
===config===
suppress=UnusedVariable
===file===
<?php

class Converter {
    public function intToString(int $x): string {
        return (string) $x;
    }
}

$c = new Converter();
$fn = $c->intToString(...);
/** @mir-check $fn is Closure(int): string */
$_ = $fn;
===expect===
