===description===
A guaranteed-to-run loop (non-empty literal foreach, do-while) does not make a
variable defined when the body assigns it on only some paths; unconditional
assignments stay defined.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foreach_if_only(int $needle): int {
    foreach ([1, 2, 3] as $item) {
        if ($item === $needle) {
            $found = $item;
        }
    }
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function foreach_if_with_break(int $needle): int {
    foreach ([1, 2, 3] as $item) {
        if ($item === $needle) {
            $found = $item;
            break;
        }
    }
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function foreach_if_with_continue(int $needle): int {
    foreach ([1, 2, 3] as $item) {
        if ($item !== $needle) {
            continue;
        }
        $found = $item;
    }
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function foreach_nested_if(int $needle): int {
    foreach ([[1], [2]] as $row) {
        foreach ($row as $item) {
            if ($item === $needle) {
                $found = $item;
            }
        }
    }
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function do_while_if_only(bool $hit, bool $more): int {
    do {
        if ($hit) {
            $found = 1;
        }
    } while ($more);
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function do_while_if_with_break(bool $more): int {
    do {
        if (rand(0, 1) === 1) {
            $found = 1;
            break;
        }
    } while ($more);
    return $found;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $found might not be defined
}

function foreach_unconditional(): int {
    foreach ([1, 2, 3] as $item) {
        $last = $item;
    }
    /** @mir-check $last is int */
    return $last;
}

function foreach_if_else_both_assign(int $needle): int {
    foreach ([1, 2, 3] as $item) {
        if ($item === $needle) {
            $found = 1;
        } else {
            $found = 2;
        }
    }
    /** @mir-check $found is int */
    return $found;
}

function foreach_assigned_before_conditional_continue(int $needle): int {
    foreach ([1, 2, 3] as $item) {
        $last = $item;
        if ($item === $needle) {
            continue;
        }
    }
    /** @mir-check $last is int */
    return $last;
}

function do_while_unconditional(bool $more): int {
    do {
        $last = 1;
    } while ($more);
    /** @mir-check $last is 1 */
    return $last;
}
===expect===
