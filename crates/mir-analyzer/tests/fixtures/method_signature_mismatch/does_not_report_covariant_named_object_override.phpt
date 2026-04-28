===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function get(): Animal { return new Animal(); }
}
class Child extends Base {
    public function get(): Cat { return new Cat(); }
}
===expect===
