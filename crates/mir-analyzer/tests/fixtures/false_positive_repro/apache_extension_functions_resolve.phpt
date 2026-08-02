===description===
FP-P16: several `apache_*` functions and `virtual` are mapped by
PhpStormStubsMap.php to `apache/apache.php`, but that stubs/ dir had no
vendored file — same missing-stub root cause as the fixed imap/ldap/ssh2/xdebug
gaps. `apache_request_headers`/`getallheaders` are excluded here since they
are already stubbed under stubs/standard/.
===config===
suppress=UnusedParam,UnusedVariable,MixedAssignment,MixedArgument
===file===
<?php

function reap(): void {
    apache_child_terminate();
}

function modules(): array {
    return apache_get_modules();
}

function version(): string|false {
    return apache_get_version();
}

function readEnv(string $name): string|false {
    return apache_getenv($name, true);
}

function lookup(string $uri): object|false {
    return apache_lookup_uri($uri);
}

function note(string $name, string $value): string {
    return apache_note($name, $value);
}

function resetTimeout(): true {
    return apache_reset_timeout();
}

function responseHeaders(): array {
    return apache_response_headers();
}

function writeEnv(string $name, string $value): true {
    return apache_setenv($name, $value, true);
}

function includeSubrequest(string $path): bool {
    return virtual($path);
}
===expect===
