===description===
Unions not allowed in p h p74
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
interface A {
}
interface B {
}
function foo (A|B $test): A&B {
    return $test;
}
