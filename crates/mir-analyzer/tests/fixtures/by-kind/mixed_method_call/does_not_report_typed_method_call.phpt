===description===
does not report typed method call
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(Foo $value): void {
    $value->bar();
}
===expect===
===ignore===
TODO
