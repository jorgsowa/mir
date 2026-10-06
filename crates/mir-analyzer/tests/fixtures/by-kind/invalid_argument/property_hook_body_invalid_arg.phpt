===description===
Invalid arguments in property hooks are analyzed.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
declare(strict_types=1);

function takesString(string $value): void {
    strlen($value);
}

final class PlainHookExample
{
    public int $value {
        get {
            takesString(123);
//                      ^^^ InvalidArgument: Argument $value of takesString() expects 'string', got '123'
            return $this->value;
        }
    }
}
