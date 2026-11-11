===description===
Under strict_types an int|float argument may be a float, which an int parameter rejects.
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

function takesInt(int $value): void {}

function run(int|float $amount, int $a, int $b): void {
    takesInt($amount);
//           ^^^^^^^ PossiblyInvalidArgument: Argument $value of takesInt() expects 'int', possibly different type 'int|float' provided
    takesInt($a / $b);
//           ^^^^^^^ PossiblyInvalidArgument: Argument $value of takesInt() expects 'int', possibly different type 'int|float' provided
}
