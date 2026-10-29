===description===
Narrowing an intersection by `instanceof` drops it when one of its parts can never also be that class (two unrelated classes, or a final class), alone or in an `||` chain.
===file===
<?php
interface Expression {}
interface Named {}
class QueryBuilder {}
class ModelBuilder {}
final class Callback {}

function unrelatedClass(Expression $v): void {
    if ($v instanceof QueryBuilder) {
        if ($v instanceof ModelBuilder) {
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            /** @trace $v */
            echo 1;
        }
    }
}

function finalClass(Expression $v): void {
    if ($v instanceof QueryBuilder) {
        if ($v instanceof Callback) {
//          ^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            /** @trace $v */
            echo 1;
        }
    }
}

function anotherInterface(Expression $v): void {
    if ($v instanceof QueryBuilder) {
        if ($v instanceof Named) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is Expression&QueryBuilder&Named
        }
    }
}

function chain(Expression $v): void {
    if ($v instanceof QueryBuilder) {
        if ($v instanceof ModelBuilder || $v instanceof Named) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is Expression&QueryBuilder&Named
        }
    }
}

function chainAllExcluded(Expression $v): void {
    if ($v instanceof QueryBuilder) {
        if ($v instanceof ModelBuilder || $v instanceof Callback) {
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            /** @trace $v */
            echo 1;
        }
    }
}
