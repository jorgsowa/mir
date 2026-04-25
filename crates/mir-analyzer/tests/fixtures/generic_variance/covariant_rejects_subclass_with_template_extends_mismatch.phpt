===file===
<?php
/** @template-covariant T */
class Box {}
class Animal {}
class Cat extends Animal {}
class Dog extends Animal {}
/** @extends Box<Cat> */
class CatBox extends Box {}
/** @param Box<Dog> $box */
function acceptsDogBox(Box $box): void { var_dump($box); }
function test(): void {
    acceptsDogBox(new CatBox());
}
===expect===
InvalidArgument: Argument $box of acceptsDogBox() expects 'Box<Dog>', got 'Box<Cat>'
