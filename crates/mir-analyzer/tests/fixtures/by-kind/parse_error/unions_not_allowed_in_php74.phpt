===description===
Unions not allowed in p h p74
===config===
php_version=7.4
===file===
<?php
interface A {
}
interface B {
}
function foo (A|B $test): A&B {
    return $test;
}
===expect===
