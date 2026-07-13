===description===
A final class checked only via `instanceof $cls` where `$cls` holds a class-string variable must not be reported unused.
===config===
suppress=
===file===
<?php
final class Widget {}

function check(object $o): bool {
    $cls = Widget::class;
    return $o instanceof $cls;
}

check(new Widget());
===expect===
