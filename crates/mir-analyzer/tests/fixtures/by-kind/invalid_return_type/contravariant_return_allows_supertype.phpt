===description===
Contravariant template param accepts a supertype in a return statement.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template-contravariant T */
class Consumer {
    /** @param T $x */
    public function take(mixed $x): void {}
}
class Animal {}
class Cat extends Animal {}

/** @return Consumer<Cat> */
function make(): Consumer {
    /** @var Consumer<Animal> $c */
    $c = new Consumer();
    return $c;
}
