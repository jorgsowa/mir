===description===
reports on class method
===file===
<?php
class Foo {
    /**
     * @return array<
//     ^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @return has unclosed generic type `array<`
     */
    public function bar(): mixed { return []; }
//                  ^^^ UndefinedDocblockClass: Docblock type 'array<' does not exist
}
