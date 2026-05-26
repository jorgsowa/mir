===description===
does not report dynamic method via variable
===file===
<?php
class Foo {
    public function hello(): void {}
}
/** @param array<string> $names */
function test(Foo $foo, array $names): void {
    foreach ($names as $name) {
        $foo->{$name}();
    }
}
===expect===
