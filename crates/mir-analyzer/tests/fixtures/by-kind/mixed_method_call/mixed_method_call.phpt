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
MixedMethodCall@9:1-9:13: Method barBar() called on mixed type
