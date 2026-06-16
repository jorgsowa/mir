===description===
preg_replace with string subject used in a function returning string — no NullableReturnStatement
when the result is coalesced (the ?string is handled)
===file===
<?php
function sanitize(string $input): string {
    return preg_replace('/[^a-z]/', '', $input) ?? '';
}
===expect===
