===description===
Array without all named parameters suppress mixed
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
    /** @suppress MixedArgument */
    return new User(...$data);
}
===expect===
UnusedSuppress@15:0-15:0: Suppress annotation for 'MixedArgument' is never used
PossiblyInvalidArgument@15:20-15:28: Argument $id of User::__construct() expects 'int', possibly different type 'int|string' provided
