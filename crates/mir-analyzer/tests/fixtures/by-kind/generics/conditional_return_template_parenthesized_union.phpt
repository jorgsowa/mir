===description===
A union of parenthesized types (`(T is Dog ? int : string)|(null)`) is split as a union instead of being read as one conditional.
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

/**
 * @template T of Animal
 * @param T $a
 * @return (T is Dog ? int : string)|(null)
 */
function legs(Animal $a) { return null; }

function test(): void {
    $a = legs(new Dog());
    /** @mir-check $a is int|null */
    echo "x$a";
    $_ = 1;
}
