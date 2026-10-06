===description===
`@return (T is Dog ? int : string)` resolves from the argument type bound to the template, including inside a union.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Cat extends Animal {}

/**
 * @template T of Animal
 * @param T $a
 * @return (T is Dog ? int : string)
 */
function legs(Animal $a) { return 4; }

/**
 * @template T of Animal
 * @param T $a
 * @return (T is Dog ? int : string)|null
 */
function legsOrNull(Animal $a) { return null; }

function test(): void {
    $a = legs(new Dog());
    /** @mir-check $a is int */
    $b = legs(new Cat());
    /** @mir-check $b is string */
    $c = legsOrNull(new Dog());
    /** @mir-check $c is int|null */
    $d = legsOrNull(new Cat());
    /** @mir-check $d is string|null */
    $_ = 1;
}
