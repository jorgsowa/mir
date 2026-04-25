===file===
<?php
/** @template-covariant T */
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
