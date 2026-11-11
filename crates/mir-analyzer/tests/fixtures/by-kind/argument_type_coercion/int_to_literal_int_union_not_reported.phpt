===description===
A plain int passed where int literals are expected is not reported: flag sets and int masks expand to such unions.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 1|2|3 $level */
function setLevel(int $level): void {}
/** @param int-mask<1, 2, 4> $flags */
function setFlags(int $flags): void {}

function run(int $input): void {
    setLevel($input);
    setFlags($input);
}
