===description===
invariant rejects subclass with template extends supertype
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                   ^^^^^^^^^^^^ InvalidArgument: Argument $box of acceptsAnimalBox() expects 'Box<Animal>', got 'Box<Cat>'
}
