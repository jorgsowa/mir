===description===
Int-backed enum ::from() rejects string argument
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum Priority: int {
    case Low = 1;
    case High = 2;
}

Priority::from('high');
//             ^^^^^^ InvalidArgument: Argument $value of from() expects 'int', got '"high"'
===expect===
