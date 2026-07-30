===description===
Negative control for the `T|callable():T` double-binding fix: a closure
whose return type genuinely violates the template's bound still flags —
the fix only stops the bare `T` alternative from ALSO absorbing the whole
closure value, it doesn't loosen the return-type structural bind.
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class Unrelated {}

/**
 * @template T of Base
 * @param T|callable():T $item
 */
function wrap($item): void {}

wrap(fn(): Unrelated => new Unrelated());
===expect===
InvalidTemplateParam@11:0-11:40: Template type 'T' inferred as 'Unrelated' does not satisfy bound 'Base'
