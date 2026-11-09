===description===
An untyped by-ref param of a method, static method or constructor may write anything, so the argument becomes mixed.
===file===
<?php
final class K {
    public function __construct(&$c) { $c = 1; }
    public function m(&$c): void { $c = 1; }
    public static function s(&$c): void { $c = 1; }
}
function f(K $k): void {
    $a = [];
    $k->m($a);
    /** @mir-check $a is mixed */
    echo 1;
    $b = [];
    K::s($b);
    /** @mir-check $b is mixed */
    echo 1;
    $c = [];
    new K($c);
    /** @mir-check $c is mixed */
    echo 1;
}
