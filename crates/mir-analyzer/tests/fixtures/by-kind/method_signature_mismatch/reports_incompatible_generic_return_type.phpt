===description===
overriding method that returns Box<int> when parent declares Box<string> should report mismatch
===ignore===
MethodSignatureMismatch only fires for native PHP return type hints, not docblock @return annotations.
Parent @return Box<string> sets from_docblock=true, which skips the check entirely (class.rs line ~397).
This is a pre-existing gap — not introduced by the generic inference fix.
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
MethodSignatureMismatch@9:4: Method Dog::make() signature mismatch: return type 'Box<int>' is not a subtype of parent 'Box<string>'
