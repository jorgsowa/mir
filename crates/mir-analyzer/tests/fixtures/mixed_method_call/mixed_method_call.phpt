===description===
Mixed method call
===file===
<?php
class Foo {
    public static function barBar(): void {}
}

/** @var mixed */
$a = (new Foo());

$a->barBar();
===expect===
MixedMethodCall
===ignore===
TODO
