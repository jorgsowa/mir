===description===
cross file since 8 0 function not defined on php 7 4
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file:StringHelper.php===
<?php
function check_contains(string $text, string $needle): void {
    str_contains($text, $needle);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function str_contains() is not defined
}
===file:App.php===
<?php
check_contains('hello world', 'world');
===expect===
