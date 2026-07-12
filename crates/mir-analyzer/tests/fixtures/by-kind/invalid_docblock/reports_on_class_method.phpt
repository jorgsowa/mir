===description===
reports on class method
===file===
<?php
class Foo {
    /**
     * @return array<
     */
    public function bar(): mixed { return []; }
}
===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @return has unclosed generic type `array<`
UndefinedDocblockClass@6:20-6:23: Docblock type 'array<' does not exist
