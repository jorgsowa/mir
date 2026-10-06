===description===
cross file inferred return type
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
