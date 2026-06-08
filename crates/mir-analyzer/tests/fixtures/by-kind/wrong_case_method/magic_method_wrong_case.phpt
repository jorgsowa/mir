===description===
Wrong case magic method name in explicit call is reported.
===file===
<?php
class Stringable2 {
    public function __toString(): string { return "x"; }
}
$s = new Stringable2();
$s->__TOSTRING();
===expect===
WrongCaseMethod@6:5-6:15: Method name 'Stringable2::__TOSTRING' has incorrect casing; use '__toString'
