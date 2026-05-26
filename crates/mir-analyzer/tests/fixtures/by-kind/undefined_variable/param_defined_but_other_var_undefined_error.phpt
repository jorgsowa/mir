===description===
param defined but other var undefined error
===file===
<?php
function transform(string $input): string {
    return $input . $suffix;
}
===expect===
UndefinedVariable@3:21: Variable $suffix is not defined
