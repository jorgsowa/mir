===description===
A write at the end of a loop body that no later iteration reads is still unused.
===config===
suppress=UnusedForeachValue,MixedAssignment
===file===
<?php
function overwritten_before_any_read(array $items): void {
    foreach ($items as $item) {
        $a = 1;
//      ^^ UnusedVariable: Variable $a is never read
        $a = null;
    }
}

function read_then_dead_trailing_write(): void {
    while (rand(0, 1)) {
        $a = rand();
        echo $a;
        $a = 1;
//      ^^ UnusedVariable: Variable $a is never read
    }
}

function nested_dead_trailing_write(array $items, array $cs): void {
    foreach ($items as $item) {
        foreach ($cs as $c) {
            $a = $c;
            echo $a;
            $a = null;
//          ^^ UnusedVariable: Variable $a is never read
        }
    }
}
===expect===
