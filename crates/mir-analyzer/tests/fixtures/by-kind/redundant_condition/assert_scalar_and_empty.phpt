===description===
Assert scalar and empty
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param mixed $value
 * @assert scalar $value
 * @assert !empty $value
 */
function assertScalarNotEmpty($value) : void {}

/** @param scalar $s */
function takesScalar($s) : void {}

/**
 * @param mixed $bar
 */
function foo($bar) : void {
    assertScalarNotEmpty($bar);
    takesScalar($bar);

    if ($bar) {}
}
===expect===
