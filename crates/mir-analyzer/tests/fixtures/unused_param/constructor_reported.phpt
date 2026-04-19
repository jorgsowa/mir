===source===
<?php
class Foo {
    public function __construct(int $x) {}
}
===expect===
UnusedParam: Parameter $x is never used
