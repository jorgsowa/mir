===description===
mixedTypeInferredIfErrors
===file===
<?php
class A {}
/**
 * @param A|string $a
 */
function foo($a): void {
    /**
     * @psalm-suppress PossiblyInvalidClone
     */
    $cloned = clone $a;
}
===expect===
MixedAssignment
===ignore===
TODO
