===description===
Basic
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_string(string $s): void { var_dump($s); }

takes_string(null);
//           ^^^^ NullArgument: Argument $s of takes_string() cannot be null
===expect===
