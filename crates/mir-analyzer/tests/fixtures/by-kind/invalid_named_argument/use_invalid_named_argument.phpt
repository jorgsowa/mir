===description===
Use invalid named argument
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
 * @param array{age: int, name: string, email: string} $input
 */
function foo(array $input) : CustomerData {
    return new CustomerData(
        aage: $input["age"],
//      ^^^^^^^^^^^^^^^^^^^ InvalidNamedArgument: CustomerData::__construct() has no parameter named $aage
        name: $input["name"],
        email: $input["email"],
    );
}
