===source===
<?php
function transform(string $input): string {
    return $input . $suffix;
}
===expect===
UndefinedVariable: Variable $suffix is not defined
