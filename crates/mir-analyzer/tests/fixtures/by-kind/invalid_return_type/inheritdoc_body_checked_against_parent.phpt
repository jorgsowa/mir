===description===
A method with @inheritdoc that returns the wrong type should still be flagged;
the parent's @return becomes the declared type for the child's body check.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
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
//      ^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"not a cat"' is not compatible with declared 'Cat'
    }
}
===expect===
