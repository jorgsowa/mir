===description===
Without strict_types a float argument is truncated to int, which is a cast warning and not an invalid argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $value): void {}

function run(float $ratio): void {
    takesInt($ratio);
//           ^^^^^^ ImplicitFloatToIntCast: Implicit cast from float to int truncates the fractional part
}
