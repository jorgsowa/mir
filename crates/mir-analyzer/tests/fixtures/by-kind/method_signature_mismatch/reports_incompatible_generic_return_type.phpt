===description===
overriding method that returns Box<int> when parent declares Box<string> should report mismatch
===file===
<?php
/** @template T */
class Box {}
class Animal {
    /** @return Box<string> */
    public function make(): mixed { return new Box(); }
}
class Dog extends Animal {
    /** @return Box<int> */
    public function make(): mixed { return new Box(); }
}
===expect===
