===description===
`!isset($arr['key'])` on a union of shapes must exclude any member where the
key is provably present and non-null (isset() would have been true there),
but only for a single-level access — a nested path's false branch doesn't
pin down which level failed, and a lone shape whose key is nullable is
already consistent with the false branch and stays unnarrowed.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
/**
 * @param array{status: int}|array{status: ?int, extra: string} $u
 */
function excludesDefinitePresentMember(array $u): void {
    if (!isset($u['status'])) {
        /** @mir-check $u is array{status: int|null, extra: string} */
        $_ = $u;
    }
}

/**
 * @param array{a: array{b: int}}|array{a: array{b: ?int, c: string}} $u
 */
function nestedPathLeftUnnarrowed(array $u): void {
    if (!isset($u['a']['b'])) {
        /** @mir-check $u is array{a: array{b: int}}|array{a: array{b: int|null, c: string}} */
        $_ = $u;
    }
}

/**
 * @param array{status: ?int} $u
 */
function loneNullableMemberUnaffected(array $u): void {
    if (!isset($u['status'])) {
        /** @mir-check $u is array{status: int|null} */
        $_ = $u;
    }
}
===expect===
