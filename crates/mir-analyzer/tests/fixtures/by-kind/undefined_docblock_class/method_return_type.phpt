===description===
A method's own `@return` docblock type referencing a nonexistent class must
report UndefinedDocblockClass, matching a free function's identical tag.
===file===
<?php
class Foo {
    /** @return UndefinedReturnClass */
    public function bar(): mixed {
//                  ^^^ UndefinedDocblockClass: Docblock type 'UndefinedReturnClass' does not exist
        return null;
    }
}
===expect===
