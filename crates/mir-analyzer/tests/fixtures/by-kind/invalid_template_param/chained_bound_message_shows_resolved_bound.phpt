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
//            ^^ UnusedParam: Parameter $t is never used
//                ^^ UnusedParam: Parameter $u is never used

pair(new Cat(), new Dog());
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'U' inferred as 'Dog' does not satisfy bound 'Cat'
===expect===
