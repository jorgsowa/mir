===description===
Int-backed enum ::tryFrom() rejects string argument
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum Priority: int {
    case Low = 1;
}

Priority::tryFrom('low');
//                ^^^^^ InvalidArgument: Argument $value of tryFrom() expects 'int', got '"low"'
