===file===
<?php
function extract(string $s): string {
    if (preg_match('/(\w+)/', $s, $matches)) {
        // $matches is populated by the by-ref stub param — must not be UndefinedVariable
        return $matches[1];
    }
    return '';
}
===expect===
