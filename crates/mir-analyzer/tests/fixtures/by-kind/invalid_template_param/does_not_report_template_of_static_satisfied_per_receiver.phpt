===description===
Regression guard alongside the T-of-static late-static-binding fix: calling
`accept()` through each receiver with an argument of that SAME receiver's
own class must not be flagged — `static` late-binds independently at each
call site, so a `Base` receiver accepting a `Base` and a `Sub` receiver
accepting a `Sub` are both valid.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Base {
    /**
     * @template T of static
     * @param T $x
     */
    public function accept($x): void {}
}
class Sub extends Base {}

$sub = new Sub();
$sub->accept(new Sub());

$base = new Base();
$base->accept(new Base());
===expect===
