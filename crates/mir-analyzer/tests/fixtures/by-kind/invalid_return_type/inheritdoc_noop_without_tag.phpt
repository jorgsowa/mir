===description===
Without @inheritdoc, a child method that overrides a parent method with a
@return docblock does NOT inherit the parent's return type. The parent's
docblock is irrelevant — no false positives in the child's body.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php
class Cat {}

abstract class Base {
    /** @return Cat */
    abstract public function make(): mixed;
}

class Child extends Base {
    public function make(): mixed {
        return "not a cat";
    }
}
===expect===
