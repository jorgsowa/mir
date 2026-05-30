===description===
not reported with nullsafe operator
===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    $obj?->bar();
}
===expect===
