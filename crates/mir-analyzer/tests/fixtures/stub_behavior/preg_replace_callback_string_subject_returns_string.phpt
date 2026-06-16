===description===
preg_replace_callback with string subject returns string|null — no NullableReturnStatement
when result is coalesced
===config===
suppress=MixedArrayAccess
===file===
<?php
function formatDate(string $date): string {
    return preg_replace_callback(
        '/(\d{4})-(\d{2})-(\d{2})/',
        fn($m) => $m[3] . '/' . $m[2] . '/' . $m[1],
        $date
    ) ?? '';
}
===expect===
