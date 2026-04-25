===file===
<?php
/** @template-contravariant T */
class Sink {
    /** @param T $v */
    public function put(mixed $v): void { var_dump($v); }
}
class Animal {}
class Cat extends Animal {}
/** @param Sink<Animal> $s */
function f(Sink $s): void { var_dump($s); }
function test(): void {
    /** @var Sink<Cat> $c */
    $c = new Sink();
    f($c);
}
===expect===
InvalidArgument: Argument $s of f() expects 'Sink<Animal>', got 'Sink<Cat>'
