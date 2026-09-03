===description===
M30: do-while's per-iteration closure never re-applied the loop condition's
own true-narrowing at the top of the next iteration, unlike the sibling
`while` (analyze_loop_widened's fixed-point merge discards narrowing between
passes unless re-injected). Re-entering the body means the previous
iteration's condition check was true, so an assignment-in-condition idiom
like `get_parent_class()`'s `!== false` loop must narrow away `false` before
the body's next pass, the same way it already does for `while`.
===file===
<?php
function walk(string $class): bool {
    do {
        if (strlen($class) > 100) {
            return true;
        }
    } while (($class = get_parent_class($class)) !== false);
    return false;
}
===expect===
