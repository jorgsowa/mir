===description===
In strict PHP (strict_types=1), returning int|false where bool is declared IS an error.
===config===
suppress=UnusedParam
===file===
<?php
declare(strict_types=1);

function isMatch(string $subject, string $pattern): bool {
    return preg_match($pattern, $subject);
}
===expect===
InvalidReturnType@5:4-5:42: Return type 'int<0, 1>|false' is not compatible with declared 'bool'
