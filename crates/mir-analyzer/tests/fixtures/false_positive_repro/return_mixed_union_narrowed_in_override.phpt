===description===
A child override narrowing a mixed object+scalar union return type (string|Animal -> string|Cat)
is covariance-legal and must not be flagged. Verifies the lifted G5 override skip stays
false-positive-free for the common narrowing direction.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function make(): string|Animal { return new Animal(); }
}
class Sub extends Base {
    public function make(): string|Cat { return new Cat(); }
}
===expect===
