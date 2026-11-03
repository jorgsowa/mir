===description===
A variable passed to a union of literal function names is premarked by every candidate's out-params.
===file===
<?php
/** @return array<array-key, mixed> */
function matchAll(string $regexp, string $subject, int $flags): array {
    $match = ($flags & \PREG_SET_ORDER) ? 'preg_match_all' : 'preg_match';
    if (false === $match($regexp, $subject, $matches, $flags)) {
        return [];
    }
    return $matches ?? [];
}
