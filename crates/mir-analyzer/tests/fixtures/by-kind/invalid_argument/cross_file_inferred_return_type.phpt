===description===
cross file inferred return type
===config===
suppress=ForbiddenCode,MissingReturnType
===file:Consumer.php===
<?php
function requireInt(int $n): void { var_dump($n); }
function test(): void {
    requireInt(getFruit());
//             ^^^^^^^^^^ InvalidArgument: Argument $n of requireInt() expects 'int', got 'Apple'
}
===file:Provider.php===
<?php
class Apple {}

function getFruit() {
    return new Apple();
}
===expect===
