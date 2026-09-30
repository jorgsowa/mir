===description===
Invalid arguments in property hooks are analyzed.
===config===
php_version=8.4
suppress=MissingConstructor
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
===expect===
