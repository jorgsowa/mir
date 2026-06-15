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
MixedMethodCall@9:0-9:12: Method barBar() called on mixed type
