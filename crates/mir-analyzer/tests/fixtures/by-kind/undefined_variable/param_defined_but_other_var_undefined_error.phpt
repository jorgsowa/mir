===description===
param defined but other var undefined error
===file===
<?php
function transform(string $input): string {
    return $input . $suffix;
//                  ^^^^^^^ UndefinedVariable: Variable $suffix is not defined
}
