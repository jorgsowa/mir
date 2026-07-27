===description===
A closure's `use (&$x)` capture of the enclosing @pure function's own
by-reference parameter never propagated byref_param_names into the
closure's own FlowState — a write to $x through the closure body mutates
the SAME caller-visible reference as a direct write in the enclosing
scope would, but was completely invisible to check_var_write_purity/
assign_to_target, both keyed off that set.
===config===
suppress=UnusedVariable
===file===
<?php
/** @pure */
function pureFn(int &$x): void {
    $f = function () use (&$x): void {
        $x = 5;
    };
    $f();
}
===expect===
ImpureByRefAssignment@5:8-5:14: Assigning to by-reference parameter $x in a @pure function
