===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(): void {
    $f = new Foo();
    $f->bar();
}
===expect===
