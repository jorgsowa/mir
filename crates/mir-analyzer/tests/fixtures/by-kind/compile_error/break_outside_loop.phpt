===description===
break and continue outside a loop or switch are compile errors.
===file===
<?php
function f(): void {
    break;
//  ^^^^^ CompileError: 'break' not in the 'loop' or 'switch' context [break-out-of-loop]
}
/** @param list<int> $xs */
function g(array $xs): void {
    foreach ($xs as $x) {
        if ($x) {
            continue;
        }
        break;
    }
}
