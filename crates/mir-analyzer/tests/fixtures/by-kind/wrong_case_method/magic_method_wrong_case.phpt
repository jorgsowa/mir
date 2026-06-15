===description===
Wrong case magic method name in explicit call is reported.
===config===
suppress=UnusedVariable
===file===
<?php
class Stringable2 {
    public function __toString(): string { return "x"; }
}
$s = new Stringable2();
$x = $s->__TOSTRING();
===expect===
WrongCaseMethod@6:9-6:19: Method name 'Stringable2::__TOSTRING' has incorrect casing; use '__toString'
