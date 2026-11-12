===description===
The defining branch may rewrite the guard variable: when it did not run, the guard is unchanged and still fails, so wherever the guard holds again the variable is defined. A write in the other branch keeps it possibly undefined, also for a guard negated into the else branch.
===file===
<?php
function load(string $uuid): ?object { return $uuid === '' ? null : new stdClass(); }

function lazyEntity(?object $entity): object {
    if (!$entity) {
        $uuid = 'default';
        $entity = load($uuid);
    }
    if (!$entity) {
        throw new RuntimeException($uuid);
    }
    return $entity;
}

function elseDefines(?object $entity): void {
    if ($entity) {
        $entity = load('other');
    } else {
        $uuid = 'default';
    }
    if (!$entity) {
        echo $uuid;
//           ^^^^^ PossiblyUndefinedVariable: Variable $uuid might not be defined
    }
}
