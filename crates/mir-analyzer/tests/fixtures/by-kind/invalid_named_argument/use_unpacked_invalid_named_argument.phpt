===description===
Use unpacked invalid named argument
===file===
<?php
class CustomerData {
    public function __construct(
        public string $name,
        public string $email,
        public int $age,
    ) {}
}

/**
 * @param array{aage: int, name: string, email: string} $input
 */
function foo(array $input) : CustomerData {
    return new CustomerData(...$input);
}
===expect===
PossiblyInvalidArgument@14:28-14:37: Argument $name of CustomerData::__construct() expects 'string', possibly different type 'int|string' provided
