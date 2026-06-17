===description===
In non-strict PHP, returning int|false (from preg_match) where bool is declared is a coercion, not InvalidReturnType.
===config===
suppress=UnusedParam
===file===
<?php
function isMatch(string $subject, string $pattern): bool {
    return preg_match($pattern, $subject);
}
===expect===
