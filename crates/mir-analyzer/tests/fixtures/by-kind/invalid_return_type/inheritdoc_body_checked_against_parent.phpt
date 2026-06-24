===description===
A method with @inheritdoc that returns the wrong type should still be flagged;
the parent's @return becomes the declared type for the child's body check.
===config===
php_version=8.2
===file===
<?php
class Cat {}

abstract class AnimalFactory {
    /** @return Cat */
    abstract public function make(): mixed;
}

class BadFactory extends AnimalFactory {
    /** @inheritdoc */
    public function make(): mixed {
        return 'not a cat';
    }
}
===expect===
InvalidReturnType@12:8-12:27: Return type '"not a cat"' is not compatible with declared 'Cat'
