===description===
A function that both applies its own opaque callback to array_map and
recurses into itself (forwarding that same opaque callback along) must
resolve via its one concrete external caller without hanging or crashing —
the recursive call site's forwarded `$cb` argument is itself unresolvable and
is simply skipped, not treated as a cycle.
===config===
suppress=MixedAssignment
===file===
<?php
function process(callable $cb, array $items): array {
    if (count($items) > 1) {
        return process($cb, array_slice($items, 1));
    }
    $result = array_map($cb, $items);
    /** @mir-check $result is array<array-key, string> */
    return $result;
}

function useIt(array $items): void {
    process(fn(int $x): string => (string) $x, $items);
}
===expect===
