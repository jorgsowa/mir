===description===
Asserting `array<string>` on an `array<string, mixed>` param makes iterating
it yield strings: no MixedAssignment/MixedArgument on the loop value.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-assert array<string> $v */
function allString(mixed $v): void {}
function takesString(string $s): void { echo $s; }

/** @param array<string, mixed> $o */
function f(array $o): void {
    allString($o);
    foreach ($o as $v) {
        takesString($v);
    }
}

/** @param array<string, mixed> $o */
function g(array $o): void {
    foreach ($o as $v) {
        takesString($v);
    }
}
===expect===
MixedAssignment@16:19-16:21: Variable $v is assigned a mixed type
MixedArgument@17:20-17:22: Argument $s of takesString() is mixed
