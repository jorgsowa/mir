===description===
Bare, property-less shape types (`object{}`, `array{}`, `list{}`) parse as
their approximated types instead of being rejected as an "empty generic
type parameter" — `{}` is the shape-literal delimiter, not a generic
argument list, so an empty one is meaningful (no known properties /
definitely-empty array), unlike an empty `<>` or `()`.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php
/** @param object{} $a */
function takesEmptyObjectShape($a): void {
    /** @mir-check $a is object */
    $_ = 1;
}

/** @param array{} $b */
function takesEmptyArrayShape($b): void {
    /** @mir-check $b is array{} */
    $_ = 1;
}

/** @param list{} $c */
function takesEmptyListShape($c): void {
    /** @mir-check $c is array{} */
    $_ = 1;
}
===expect===
