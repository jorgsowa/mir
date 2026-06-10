===description===
Intersections not allowed with non classes
===file===
<?php
interface A {
}
function foo (A&string $test): A&string {
    return $test;
}
===expect===
