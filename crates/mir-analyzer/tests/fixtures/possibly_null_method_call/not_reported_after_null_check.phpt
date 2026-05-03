===description===
not reported after null check
===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    if ($obj !== null) {
        $obj->bar();
    }
}
===expect===
===ignore===
TODO
