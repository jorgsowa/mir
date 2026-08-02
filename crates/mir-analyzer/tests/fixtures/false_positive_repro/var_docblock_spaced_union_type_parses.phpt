===description===
`@var string | null $x` (spaces around `|`) failed to parse as a named annotation:
`parse_var_line` bailed out (`return None`) at the first depth-0 whitespace unless it
was immediately followed by `$name`, so it gave up right after `string`, before
reaching ` | null $x`. The caller then fell back to treating it as a bare, nameless
annotation using only the truncated `string` prefix, silently dropping the `|null`
half of the union — so a later `$x !== null` check was wrongly reported as always
true. Fixed by letting the scan continue across a depth-0 whitespace that borders a
`|`/`&` (on either side), not just stop at the first non-`$name` token.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo | null $x */
    $x = null;
    if ($x !== null) {
        $x->bar();
    }
}
===expect===
