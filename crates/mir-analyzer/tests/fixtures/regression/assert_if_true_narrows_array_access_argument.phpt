===description===
`@psalm-assert-if-true string $arr['key']` targets a specific key of the
`$arr` parameter, not the whole parameter — the assertion's target name
was extracted verbatim including the bracket suffix (`"arr['key']"`),
which could never match a real declared parameter name, so the whole
assertion silently no-oped for this shape.
===config===
suppress=MissingReturnType,MixedArgument
===file===
<?php
/**
 * @param array{status?: string} $config
 * @psalm-assert-if-true string $config['status']
 */
function hasStatus(array $config) {
    return isset($config['status']);
}

/** @param array{status?: string} $c */
function test(array $c): void {
    if (hasStatus($c)) {
        /** @mir-check $c is array{status: string} */
        $_ = 1;
    }
}
===expect===
