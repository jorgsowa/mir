===description===
Before PHP 8.4 a reserved enum name reads as a reserved class name.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
enum parent: int {
//   ^^^^^^ CompileError: Cannot use 'parent' as class name as it is reserved [reserved-enum-name]
    case A = 1;
}
