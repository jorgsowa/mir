===description===
covariant rejects supertype arg
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}
/** @param Box<Cat> $b */
function f(Box $b): void { var_dump($b->get()); }
function test(): void {
    /** @var Box<Animal> $a */
    $a = new Box();
    f($a);
//    ^^ InvalidArgument: Argument $b of f() expects 'Box<Cat>', got 'Box<Animal>'
}
===expect===
