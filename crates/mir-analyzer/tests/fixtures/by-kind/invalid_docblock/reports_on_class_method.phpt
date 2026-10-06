===description===
reports on class method
===file===
<?php
class Foo {
    /**
     * @return array<
     */
    public function bar(): mixed { return []; }
//                  ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
}
===expect===
InvalidDocblock@4:7-4:21: Invalid docblock: @return has unclosed generic type `array<`
