===description===
An enum method implementing an interface method must be checked for
signature compatibility (return type, param narrowing) just like a class —
enum method signatures were never compared against the interface at all,
only checked for name presence.
===file===
<?php
interface Greeter {
    public function greet(Animal $a): string;
}
class Animal {}
class Dog extends Animal {}

enum Status implements Greeter {
    case Active;
    case Inactive;
    public function greet(Dog $a): string { return "hi"; }
}
===expect===
MethodSignatureMismatch@11:4-11:58: Method Status::greet() signature mismatch: parameter $a type 'Dog' is narrower than parent type 'Animal'
UnusedParam@11:26-11:32: Parameter $a is never used
