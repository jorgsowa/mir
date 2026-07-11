===description===
Regression guard alongside the template-bound-checking fix: a call through
an instance method's first-class-callable closure that DOES satisfy the
method's own `@template T of Base` bound (a `Sub extends Base` argument)
must not be flagged — only an actual bound violation should be.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Base {}
class Sub extends Base {}
class Box {
    /**
     * @template T of Base
     * @param T $item
     */
    public function put($item): void {}
}

$box = new Box();
$fn = $box->put(...);
$fn(new Sub());
===expect===
