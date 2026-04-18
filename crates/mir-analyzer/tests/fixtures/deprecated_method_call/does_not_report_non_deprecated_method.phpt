===source===
<?php
class Foo {
    public function activeMethod(): void {}
}

$foo = new Foo();
$foo->activeMethod();
===expect===
