===description===
Under strict_types an int still widens to float, and a float fits float, int|float and mixed parameters.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function takesFloat(float $value): void {}
function takesNumber(int|float $value): void {}
function takesMixed(mixed $value): void {}
function takesInt(int $value): void {}

function run(float $ratio, int $count): void {
    takesFloat($count);
    takesFloat($ratio);
    takesNumber($ratio);
    takesMixed($ratio);
    takesInt((int) $ratio);
    takesInt($count * 2);
}
