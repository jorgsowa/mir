===description===
Companion to the method_signature_mismatch fixture for the same fix: a
compatible override against a class-bound template must not be flagged, and
a subclass that never binds the template at all (no `@extends Box<...>`
type argument) must keep the old "can't check statically" skip rather than
gaining a new false positive.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $x */
    public function set($x): void {}

    /** @return T */
    public function get() {
        return null;
    }
}

/** @extends Box<int> */
class IntBox extends Box {
    public function set(int $x): void {}
    public function get(): int {
        return 1;
    }
}

class UnboundSub extends Box {
    /** @param mixed $x */
    public function set($x): void {}
}
===expect===
