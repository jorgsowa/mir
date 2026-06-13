===description===
DeprecatedMethodCall does NOT fire for methods without @deprecated or #[Deprecated].
===file===
<?php
class Foo {
    public static function current(): void {}
}

Foo::current();
===expect===
