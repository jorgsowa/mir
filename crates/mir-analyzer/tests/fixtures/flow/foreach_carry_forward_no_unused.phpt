===description===
A carry-forward variable (written at end of loop body, read at start of next iteration)
must not be reported as UnusedVariable. The last iteration's write is never read again
after the loop, but that is unavoidable from the developer's perspective.
===config===
suppress=MixedAssignment,RedundantCondition,NullArrayAccess,MixedArgument,PossiblyNullArrayAccess
===file===
<?php
function before(array $items): mixed {
    $previous = null;
    foreach ($items as $item) {
        if ($previous !== null) {
            echo $previous;
        }
        $previous = $item;
    }
    return null;
}

function carryArray(array $columns): void {
    $previous = null;
    foreach ($columns as $column) {
        $merged = array_merge([$column], $previous['cols'] ?? []);
        echo implode(',', $merged);
        $previous = ['cols' => $merged];
    }
}
===expect===
