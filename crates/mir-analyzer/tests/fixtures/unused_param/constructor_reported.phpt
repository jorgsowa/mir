===description===
constructor reported
===file===
<?php
class Foo {
    public function __construct(int $x) {}
}
===expect===
UnusedParam@3:33: Parameter $x is never used
