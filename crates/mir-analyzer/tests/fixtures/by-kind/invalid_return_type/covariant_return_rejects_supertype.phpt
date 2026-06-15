===description===
Covariant template param still rejects an unrelated sibling type in a
return statement (Dog and Cat both extend Animal but are unrelated to
each other, so neither covariance direction saves them).
===file===
<?php
/** @template-covariant T */
class Box {
    /** @return T */
    public function get(): mixed { return null; }
}
class Animal {}
class Cat extends Animal {}
class Dog extends Animal {}

/** @return Box<Cat> */
function make(): Box {
    /** @var Box<Dog> $b */
    $b = new Box();
    return $b;
}
===expect===
InvalidReturnType@15:4-15:14: Return type 'Box<Dog>' is not compatible with declared 'Box<Cat>'
