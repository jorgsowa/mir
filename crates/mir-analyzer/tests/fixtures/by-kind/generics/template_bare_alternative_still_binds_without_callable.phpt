===description===
Regression control for the `T|callable():T` double-binding fix: passing a
plain `T` value (not a closure) through the same union-typed param must
still bind the bare `T` alternative normally.
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class Concrete extends Base {}

/**
 * @template T of Base
 * @param T|callable():T $item
 */
function wrap($item): void {}

wrap(new Concrete());
===expect===
