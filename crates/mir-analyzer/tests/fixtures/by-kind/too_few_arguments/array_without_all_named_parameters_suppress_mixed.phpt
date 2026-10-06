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
UnusedSuppress@14:18-14:31: Suppress annotation for 'MixedArgument' is never used
