===description===
Mixed type inferred if errors
===config===
suppress=UnusedVariable
===file===
<?php
class A {}
/**
 * @param A|string $a
 */
function foo($a): void {
    /**
     * @suppress PossiblyInvalidClone
     */
    $cloned = clone $a;
}
===expect===
