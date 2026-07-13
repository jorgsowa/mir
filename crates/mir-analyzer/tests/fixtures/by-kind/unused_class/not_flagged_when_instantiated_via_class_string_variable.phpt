===description===
A final class instantiated only through `new $class()` where `$class` holds a class-string variable must not be reported unused.
===config===
suppress=
===file===
<?php
final class Widget {}

function make(): Widget {
    $cls = Widget::class;
    return new $cls();
}

make();
===expect===
