===description===
cross file narrowing param type
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Animal.php===
<?php
class Animal {
    public function eat(string $food): void { var_dump($food); }
}
===file:Dog.php===
<?php
class Dog extends Animal {
    public function eat(int $food): void { var_dump($food); }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Dog::eat() signature mismatch: parameter $food type 'int' is incompatible with parent type 'string'
}
