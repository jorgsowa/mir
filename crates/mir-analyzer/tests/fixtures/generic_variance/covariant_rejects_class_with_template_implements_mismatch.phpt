===file===
<?php
/** @template-covariant T */
interface Source {}
class Animal {}
class Cat extends Animal {}
class Dog extends Animal {}
/** @implements Source<Cat> */
class CatSource implements Source {}
/** @param Source<Dog> $source */
function acceptsDogSource(Source $source): void { var_dump($source); }
function test(): void {
    acceptsDogSource(new CatSource());
}
===expect===
InvalidArgument: Argument $source of acceptsDogSource() expects 'Source<Dog>', got 'Source<Cat>'
