===description===
No named args method
===file===
<?php
class CustomerData
{
    /** @no-named-arguments */
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
        age: $input["age"],
//      ^^^^^^^^^^^^^^^^^^ InvalidNamedArguments: CustomerData::__construct() does not accept named arguments
        name: $input["name"],
//      ^^^^^^^^^^^^^^^^^^^^ InvalidNamedArguments: CustomerData::__construct() does not accept named arguments
        email: $input["email"],
//      ^^^^^^^^^^^^^^^^^^^^^^ InvalidNamedArguments: CustomerData::__construct() does not accept named arguments
    );
}
