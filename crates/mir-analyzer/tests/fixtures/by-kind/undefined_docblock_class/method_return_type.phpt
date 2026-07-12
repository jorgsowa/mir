===description===
A method's own `@return` docblock type referencing a nonexistent class must
report UndefinedDocblockClass, matching a free function's identical tag.
===file===
<?php
class Foo {
    /** @return UndefinedReturnClass */
    public function bar(): mixed {
        return null;
    }
}
===expect===
UndefinedDocblockClass@4:20-4:23: Docblock type 'UndefinedReturnClass' does not exist
