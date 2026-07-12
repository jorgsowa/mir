===description===
A method's own `@param` docblock type referencing a nonexistent class must
report UndefinedDocblockClass, matching a free function's identical tag.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    /** @param UndefinedParamClass $x */
    public function bar($x): void {}
}
===expect===
UndefinedDocblockClass@4:20-4:23: Docblock type 'UndefinedParamClass' does not exist
