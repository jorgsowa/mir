===file===
<?php
class Foo {
    /**
     * @return array<
     */
    public function bar(): mixed { return []; }
}
===expect===
InvalidDocblock: Invalid docblock: @return has unclosed generic type `array<`
