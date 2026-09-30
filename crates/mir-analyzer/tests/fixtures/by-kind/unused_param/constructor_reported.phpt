===description===
constructor reported
===file===
<?php
class Foo {
    public function __construct(int $x) {}
//                              ^^^^^^ UnusedParam: Parameter $x is never used
}
===expect===
