===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(Foo $value): void {
    $value->bar();
}
===expect===
