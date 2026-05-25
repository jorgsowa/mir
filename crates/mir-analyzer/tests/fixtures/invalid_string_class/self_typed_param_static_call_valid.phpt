===description===
Variable typed as self used with :: should not emit InvalidStringClass
===file===
<?php
class Foo {
    public static function bar(): void {}

    public function test(self $other): void {
        $other::bar();
    }
}
===expect===
