===description===
glob() returns list<string>|false
===file===
<?php
/** @return list<string> */
function dirs(string $pattern): array {
    $found = glob($pattern);
    /** @mir-check $found is list<string>|false */
    return $found ?: [];
}
