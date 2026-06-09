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
InvalidDocblock@6:0-6:0: Invalid docblock: @method parameter `&$a` uses by-reference (`&`) which is not supported in @method annotations
