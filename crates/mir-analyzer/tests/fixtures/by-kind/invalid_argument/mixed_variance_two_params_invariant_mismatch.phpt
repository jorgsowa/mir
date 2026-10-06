===description===
mixed variance two params invariant mismatch
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template K
 * @template-covariant V
 */
class Pair {}
class Animal {}
class Cat extends Animal {}
/** @param Pair<string, Animal> $p */
function f(Pair $p): void { var_dump($p); }
function test(): void {
    /** @var Pair<int, Cat> $p */
    $p = new Pair();
    f($p);
//    ^^ InvalidArgument: Argument $p of f() expects 'Pair<string, Animal>', got 'Pair<int, Cat>'
}
