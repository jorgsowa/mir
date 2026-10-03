===description===
stub file function type checked
===config===
<mir>
  <stubs>
    <file name="stubs/helpers.php"/>
  </stubs>
</mir>
===file:stubs/helpers.php===
<?php
function my_helper(string $s): string { return $s; }
===file:App.php===
<?php
function test(): void { my_helper(42); }
//                                ^^ ArgumentTypeCoercion: Argument $s of my_helper() expects 'string', got '42' — coercion may fail at runtime
===expect===
