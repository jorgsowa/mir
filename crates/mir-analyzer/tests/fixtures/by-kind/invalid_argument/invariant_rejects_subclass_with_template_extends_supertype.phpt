===description===
invariant rejects subclass with template extends supertype
===file===
<?php
/** @template T */
class Box {}
class Animal {}
class Cat extends Animal {}
/** @extends Box<Cat> */
class CatBox extends Box {}
/** @param Box<Animal> $box */
function acceptsAnimalBox(Box $box): void { var_dump($box); }
function test(): void {
    acceptsAnimalBox(new CatBox());
}
===expect===
InvalidArgument@11:22-11:34: Argument $box of acceptsAnimalBox() expects 'Box<Animal>', got 'Box<Cat>'
