===description===
cross file inferred return type
===file:Consumer.php===
<?php
function requireInt(int $n): void { var_dump($n); }
function test(): void {
    requireInt(getFruit());
}
===file:Provider.php===
<?php
class Apple {}

function getFruit() {
    return new Apple();
}
===expect===
Consumer.php: InvalidArgument@4:16: Argument $n of requireInt() expects 'int', got 'Apple'
