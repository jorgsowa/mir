===description===
Laravel FP (laravel/framework): `is_object($x) ? clone $x : $x` is guarded, but
mir does not narrow on is_object() and flags MixedClone on the clone of a mixed
value. Ignored pending fix — see ROADMAP §1.4 (is_object narrowing).
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedAssignment
===file===
<?php
/** @param mixed $event */
function dispatch($event): mixed {
    return is_object($event) ? clone $event : $event;
}
===expect===
