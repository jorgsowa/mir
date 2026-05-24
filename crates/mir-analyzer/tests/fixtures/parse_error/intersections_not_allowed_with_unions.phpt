===description===
intersectionsNotAllowedWithUnions
===file===
<?php
interface A {
}
interface B {
}
interface C {
}
function foo (A&B|C $test): A&B|C {
    return $test;
}
===expect===
ParseError
===ignore===
TODO
