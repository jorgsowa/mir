===description===
Annotation with by ref param
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method string getString(&$a)
 */
class Child extends ParentClass {}
===expect===
InvalidDocblock
===ignore===
TODO
