===description===
Mixed type inferred if errors
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
