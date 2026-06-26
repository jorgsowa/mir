===description===
MissingReturnType does NOT fire for regular class methods — only interface methods
and top-level functions are checked.
===file===
<?php
class Foo {
    public function noReturn() { return 1; }
    public static function staticNoReturn() { return 2; }
    protected function protectedNoReturn() { return 3; }
}
===expect===
