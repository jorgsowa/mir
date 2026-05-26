===description===
Annotation with bad docblock
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method string getString()
 */
class Child extends ParentClass {}
===expect===
InvalidDocblock
===ignore===
TODO
