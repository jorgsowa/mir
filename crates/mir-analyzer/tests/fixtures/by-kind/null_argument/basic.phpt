===description===
Basic
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_string(string $s): void { var_dump($s); }

takes_string(null);
===expect===
NullArgument@4:14-4:18: Argument $s of takes_string() cannot be null
