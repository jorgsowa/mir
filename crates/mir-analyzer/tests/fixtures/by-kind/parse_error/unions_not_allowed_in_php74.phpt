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
//             ^ ParseError: 'union types' requires PHP 8.0 or higher [version-too-low]
//                         ^ ParseError: 'intersection types' requires PHP 8.1 or higher [version-too-low]
    return $test;
}
