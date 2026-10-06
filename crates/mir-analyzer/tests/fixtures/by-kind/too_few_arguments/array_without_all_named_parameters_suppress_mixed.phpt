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
//                ^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'MixedArgument' is never used
    return new User(...$data);
}
