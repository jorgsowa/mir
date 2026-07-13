===description===
`continue 2;` inside a switch nested in a loop correctly targets the outer
loop (skips straight to the next iteration), not the switch — so the
code after the switch is only ever reached via the `default` arm, which
always sets $x.
===config===
suppress=UnusedParam,MixedAssignment
===file===
<?php
function f(array $items): void {
    foreach ($items as $item) {
        switch ($item) {
            case 1:
                continue 2;
            default:
                $x = 1;
        }
        echo $x;
    }
}
===expect===
