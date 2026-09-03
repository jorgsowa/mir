===description===
A `T|callable():T` docblock param (the standard "value or a factory for the
value" idiom): passing a closure must bind `T` from the closure's RETURN
type alone. The bare `T` alternative in the union also matched the whole
closure argument (no filter rule recognized a `callable():T`/`Closure` pair
as "covered"), merging a bogus `Closure(): X` binding into the same slot as
the correct `X` binding from the return-type structural match, so `T`
resolved to `Closure(): X|X` and failed its own bound check.
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

wrap(fn(): Concrete => new Concrete());
===expect===
