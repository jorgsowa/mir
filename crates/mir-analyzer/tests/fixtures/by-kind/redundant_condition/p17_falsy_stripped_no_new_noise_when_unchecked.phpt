===description===
FP-P17 negative control: code that does NOT defensively check preg_split /
preg_replace_callback / mb_convert_encoding's result must keep getting the
narrowed (non-|false / non-|null) type with no new diagnostics — the
falsy_stripped exemption must only affect defensive-check impossibility/
redundancy checks, not everyday unchecked usage.
===config===
suppress=UnusedParam,MixedArrayAccess
===file===
<?php

function split_into_param(string $pattern, string $subject): array {
    return preg_split($pattern, $subject);
}

function replace_callback_into_param(string $pattern, string $subject): string {
    return preg_replace_callback($pattern, fn($m) => $m[0], $subject);
}

function convert_into_param(string $s): string {
    return mb_convert_encoding($s, 'UTF-8', 'ISO-8859-1');
}
===expect===
