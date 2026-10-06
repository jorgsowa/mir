===description===
Invalid arguments in promoted-property hooks are analyzed.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
declare(strict_types=1);

function takesString(string $value): void {
    strlen($value);
}

final class PromotedHookExample
{
    public function __construct(
        public int $value {
            get {
                return $this->value;
            }
            set {
                takesString($value);
//                          ^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got 'int'
            }
        },
        public int $other {
            get {
                takesString($this->other);
//                          ^^^^^^^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got 'int'
                return $this->other;
            }
        },
    ) {}
}
