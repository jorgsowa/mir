===description===
A returned literal is checked against a `lowercase-string` or `uppercase-string` return type.
===file===
<?php
/** @return lowercase-string */
function defaultLocale(): string {
    return 'en_US';
//  ^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"en_US"' is not compatible with declared 'lowercase-string'
}

/** @return lowercase-string */
function defaultScheme(): string {
    return 'https';
}

/** @return uppercase-string */
function defaultMethod(bool $safe): string {
    return $safe ? 'GET' : 'post';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"GET"|"post"' is not compatible with declared 'uppercase-string'
}

/** @return non-empty-uppercase-string */
function currency(string $code): string {
    return strtoupper($code . 'x');
}
