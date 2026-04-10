===source===
<?php
/** @template-contravariant T */
class Sink {
    /** @param T $v */
    public function put(mixed $v): void { var_dump($v); }
}
class Animal {}
class Cat extends Animal {}
/** @param Sink<Cat> $s */
function f(Sink $s): void { var_dump($s); }
function test(): void {
    /** @var Sink<Animal> $a */
    $a = new Sink();
    f($a);
}
===expect===
