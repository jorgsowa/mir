===description===
Intersections not allowed in p h p80
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
interface A {
}
interface B {
}
function foo (A&B $test): A&B {
//             ^ ParseError: 'intersection types' requires PHP 8.1 or higher [version-too-low]
//                         ^ ParseError: 'intersection types' requires PHP 8.1 or higher [version-too-low]
    return $test;
}
