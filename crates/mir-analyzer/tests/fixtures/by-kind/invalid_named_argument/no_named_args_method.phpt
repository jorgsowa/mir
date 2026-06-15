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
        name: $input["name"],
        email: $input["email"],
    );
}
===expect===
InvalidNamedArguments@17:8-17:26: CustomerData::__construct() does not accept named arguments
InvalidNamedArguments@18:8-18:28: CustomerData::__construct() does not accept named arguments
InvalidNamedArguments@19:8-19:30: CustomerData::__construct() does not accept named arguments
