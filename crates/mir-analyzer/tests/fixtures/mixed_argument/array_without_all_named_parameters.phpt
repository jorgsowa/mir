===description===
arrayWithoutAllNamedParameters
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
MixedArgument
===ignore===
TODO
