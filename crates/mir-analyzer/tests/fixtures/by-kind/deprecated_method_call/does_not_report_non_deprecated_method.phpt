===description===
does not report non deprecated method
===file===
<?php
class Foo {
    public function activeMethod(): void {}
}

$foo = new Foo();
$foo->activeMethod();
===expect===
