===description===
FP-I4: getenv($name) with a non-null $name was typed
non-empty-string|non-empty-array|"default" (the stub's array-of-all-vars
branch leaking in), so passing the `?:` fallback into a string-only function
flagged PossiblyInvalidArgument. A literal/non-null $name can never hit the
no-args array branch.
===config===
===file===
<?php

function get_setting(): string {
    return getenv('APP_ENV') ?: 'production';
}

function use_setting(): void {
    echo strtoupper(get_setting());
}

function get_setting_with_local_only(string $name): string {
    return getenv($name, true) ?: 'default';
}
===expect===
