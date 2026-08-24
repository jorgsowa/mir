===description===
invalid argument inside constructor-promoted property hook bodies is analyzed
===config===
php_version=8.4
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
            }
        },
        public int $other {
            get {
                takesString($this->other);
                return $this->other;
            }
        },
    ) {}
}
===expect===
InvalidArgument@16:28-16:34: Argument $value of takesString() expects 'string', got 'int'
InvalidArgument@21:28-21:40: Argument $value of takesString() expects 'string', got 'int'
