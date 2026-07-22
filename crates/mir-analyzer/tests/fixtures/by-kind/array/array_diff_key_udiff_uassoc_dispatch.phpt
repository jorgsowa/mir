===description===
array_diff_key and array_udiff_uassoc are non-representative members of the
diff/intersect family (key-only comparison; user-callback comparison of both
key and value) — proves the shared return-type helper's dispatch wiring
reaches these variants too, not just array_diff/array_intersect themselves.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param array<string, int> $assoc
 */
function test(array $assoc): void {
    $diff_key = array_diff_key($assoc, ['x' => 1]);
    /** @mir-check $diff_key is array<string, int> */
    $_ = $diff_key;

    $udiff_uassoc = array_udiff_uassoc(
        $assoc,
        ['x' => 1],
        fn (int $a, int $b): int => $a <=> $b,
        fn (string $a, string $b): int => $a <=> $b,
    );
    /** @mir-check $udiff_uassoc is array<string, int> */
    $_ = $udiff_uassoc;
}
===expect===
