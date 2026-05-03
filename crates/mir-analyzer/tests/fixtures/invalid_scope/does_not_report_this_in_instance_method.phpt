===description===
does not report this in instance method
===file===
<?php
class Foo {
    public function bar(): void {
        $this->baz();
    }
    public function baz(): void {}
}
===expect===
===ignore===
TODO
