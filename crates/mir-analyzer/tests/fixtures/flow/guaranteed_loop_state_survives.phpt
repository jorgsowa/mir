===description===
Loop-state merging must not invent a zero-iteration path for loops that are
known to execute. Assignments and nullability refinements made by a do-while
or non-empty foreach body remain valid after the loop.
===config===
suppress=ImpossibleIdenticalComparison,RedundantCondition
===file===
<?php
class Box {
    public function ping(): void {}
}

function doAssignment(?Box $box, bool $keepGoing): void {
    do {
        $box = new Box;
    } while ($keepGoing);

    $box->ping();
}

/** @param non-empty-list<int> $items */
function foreachAssignment(array $items, ?Box $box): void {
    foreach ($items as $_) {
        $box = new Box;
    }

    $box->ping();
}

function doRefinement(?Box $box, bool $keepGoing): void {
    do {
        if ($box === null) {
            return;
        }
    } while ($keepGoing);

    $box->ping();
}
===expect===
