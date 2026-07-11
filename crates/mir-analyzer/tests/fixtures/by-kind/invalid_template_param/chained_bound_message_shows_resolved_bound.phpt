===description===
`@template U of T` where T is itself bound from another argument: the
violation message names the concrete resolved bound (e.g. 'Cat', the type T
was actually inferred as at this call site) instead of the raw, unresolved
template name 'T'.
===file:test.php===
<?php
class Base {}
class Cat extends Base {}
class Dog extends Base {}

/**
 * @template T of Base
 * @template U of T
 * @param T $t
 * @param U $u
 */
function pair($t, $u): void {}

pair(new Cat(), new Dog());
===expect===
test.php: UnusedParam@12:14-12:16: Parameter $t is never used
test.php: UnusedParam@12:18-12:20: Parameter $u is never used
test.php: InvalidTemplateParam@14:0-14:26: Template type 'U' inferred as 'Dog' does not satisfy bound 'Cat'
