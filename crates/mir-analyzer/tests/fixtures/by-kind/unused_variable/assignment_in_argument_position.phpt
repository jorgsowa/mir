===description===
An assignment expression in argument position (function, method, static and
constructor calls) has its value consumed by the call — not UnusedVariable
even without a later read. A plain unused assignment still reports.
===file===
<?php
class Mock {
    public function m($v): void {}
    public static function s($v): void {}
}
class Box { public function __construct($v) {} }
function f($v) { return $v; }

function args_consume(Mock $k): void {
    f($a = 1);
    $k->m($b = 2);
    Mock::s($c = 3);
    $box = new Box($d = 4);
    f($box);
}

function still_unused(): void {
    $o = new Mock();
}
===expect===
UnusedVariable@18:5-18:7: Variable $o is never read
