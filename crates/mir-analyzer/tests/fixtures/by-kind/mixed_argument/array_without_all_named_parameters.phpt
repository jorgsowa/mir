===description===
Array without all named parameters
===file===
<?php
class User {
    public function __construct(
        public int $id,
        public string $name,
        public int $age
    ) {}
}

/**
 * @param array{id: int, name: string} $data
 */
function processUserDataInvalid(array $data) : User {
    return new User(...$data);
}
===expect===
PossiblyInvalidArgument@14:20-14:28: Argument $id of User::__construct() expects 'int', possibly different type 'int|string' provided
