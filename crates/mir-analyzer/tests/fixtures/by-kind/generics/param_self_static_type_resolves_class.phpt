===description===
`@param self $x` / `@param static $x` docblock param types stored a
permanently-unresolvable empty-fqcn `TSelf`/`TStaticObject` sentinel —
every other resolved-type slot (return type, `@if-this-is`,
`@psalm-self-out`) ran the result through `fill_self_static_parent` to
plug the declaring class in, but the param path didn't, so a call site
passing a real instance of the declaring class got a bogus InvalidArgument.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Point {
    /** @param self $other */
    public function distanceTo($other): void {}

    /** @param static $other */
    public function sameClassAs($other): void {}
}

$p = new Point();
$q = new Point();
$p->distanceTo($q);
$p->sameClassAs($q);
===expect===
