===description===
unionsNotAllowedInPHP74
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
ParseError
===ignore===
TODO
