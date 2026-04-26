===file:Limits.php===
<?php
function getMaxId(): int {
    return PHP_INT_MAX;
}

function getMinId(): int {
    return PHP_INT_MIN;
}
===file:Validator.php===
<?php
function isValidId(int $id): bool {
    return $id > 0 && $id < PHP_INT_MAX;
}
===file:Main.php===
<?php
$max = getMaxId();
$ok = isValidId(99);
===expect===
