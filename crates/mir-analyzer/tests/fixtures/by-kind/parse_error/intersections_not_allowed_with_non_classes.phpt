===description===
Intersections not allowed with non classes
===ignore===
TODO
===file===
<?php
interface A {
}
function foo (A&string $test): A&string {
    return $test;
}
===expect===
