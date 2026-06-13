===description===
Regression (laravel/framework): `is_object($x) ? clone $x : $x` is guarded.
is_object() now narrows a mixed value to `object`, so the clone in the guarded
branch no longer flags MixedClone.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedAssignment
===file===
<?php
/** @param mixed $event */
function dispatch($event): mixed {
    return is_object($event) ? clone $event : $event;
}
===expect===
