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
InvalidNamedArguments@17:9-17:27: CustomerData::__construct() does not accept named arguments
InvalidNamedArguments@18:9-18:29: CustomerData::__construct() does not accept named arguments
InvalidNamedArguments@19:9-19:31: CustomerData::__construct() does not accept named arguments
