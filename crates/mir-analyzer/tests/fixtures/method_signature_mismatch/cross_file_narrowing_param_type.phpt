===file:Animal.php===
<?php
class Animal {
    public function eat(string $food): void { var_dump($food); }
}
===file:Dog.php===
<?php
class Dog extends Animal {
    public function eat(int $food): void { var_dump($food); }
}
===expect===
Dog.php: MethodSignatureMismatch: Method Dog::eat() signature mismatch: parameter $food type 'int' is narrower than parent type 'string'
