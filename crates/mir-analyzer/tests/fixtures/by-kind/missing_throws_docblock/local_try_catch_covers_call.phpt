===description===
M20: a local try/catch that covers a callee's declared @throws (or a
direct throw) makes MissingThrowsDocblock unnecessary — the exception
never escapes this function. A catch that doesn't cover the thrown type
(wrong type, or only covered by an unrelated sibling catch) still flags,
and coverage accumulates through nested try/catch (an outer catch still
counts even if a nested inner catch doesn't cover it).
===config===
suppress=UnusedVariable
===file===
<?php
/** @throws \Exception */
function risky(): void {
    throw new \Exception('boom');
}

function directThrowCaught(): void {
    try {
        throw new \Exception('stop');
    } catch (\Exception $e) {
    }
}

function callCaught(): void {
    try {
        risky();
    } catch (\Exception $e) {
    }
}

function callNotCaught(): void {
    try {
        risky();
    } catch (\TypeError $e) {
    }
}

function callCaughtBySiblingOnly(): void {
    try {
        risky();
    } catch (\TypeError $e) {
    } catch (\ValueError $e) {
    }
}

function callCaughtByOuterNestedTry(): void {
    try {
        try {
            risky();
        } catch (\TypeError $e) {
        }
    } catch (\Exception $e) {
    }
}
===expect===
MissingThrowsDocblock@23:8-23:15: Exception Exception is thrown but not declared in @throws
MissingThrowsDocblock@30:8-30:15: Exception Exception is thrown but not declared in @throws
