===file===
<?php
class Animal {}
class Dog {}
class Base {
    public function get(): Animal { return new Animal(); }
}
class Child extends Base {
    public function get(): Dog { return new Dog(); }
}
===expect===
MethodSignatureMismatch: Method Child::get() signature mismatch: return type 'Dog' is not a subtype of parent 'Animal'
