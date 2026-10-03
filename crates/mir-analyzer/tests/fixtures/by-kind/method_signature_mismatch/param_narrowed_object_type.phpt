===description===
G4: a child illegally narrows an object param from Animal to Cat (contravariance
violation) — must emit MethodSignatureMismatch.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function feed(Animal $a): void {}
}
class Kitten extends Base {
    public function feed(Cat $a): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Kitten::feed() signature mismatch: parameter $a type 'Cat' is narrower than parent type 'Animal'
}
===expect===
