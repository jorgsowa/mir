===description===
Psalm's names for the file, include, eval and unserialize sinks suppress them.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sinks(): void {
    /** @psalm-suppress TaintedFile */
    $a = file_get_contents($_GET['a']);
    /** @psalm-suppress TaintedInclude */
    include $_GET['b'];
    /** @psalm-suppress TaintedEval */
    eval($_GET['c']);
    /** @psalm-suppress TaintedUnserialize */
    $d = unserialize($_GET['d']);
}
