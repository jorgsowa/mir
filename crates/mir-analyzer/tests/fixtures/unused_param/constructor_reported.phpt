===description===
constructor reported
===file===
<?php
class Foo {
    public function __construct(int $x) {}
}
===expect===
UnusedParam@3:32: Parameter $x is never used
===ignore===
TODO
