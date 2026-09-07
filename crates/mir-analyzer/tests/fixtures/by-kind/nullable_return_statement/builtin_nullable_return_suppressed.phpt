===description===
FP-C: builtin functions like preg_replace that return string|null must not
emit NullableReturnStatement when the caller returns the result as string —
the null branch only fires on programming error (invalid regex), which is
not modeled at compile time.
===config===
php_version=8.2
===file===
<?php

function clean(string $input): string {
    return preg_replace('/\s+/', ' ', $input);
}
===expect===
