===description===
Intersections not allowed in p h p80
===config===
php_version=8.0
===file===
<?php
interface A {
}
interface B {
}
function foo (A&B $test): A&B {
    return $test;
}
===expect===
