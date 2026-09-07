===description===
@psalm-type alias used in @return on a standalone function does not produce false positives
===file===
<?php
namespace App;

class User {
    public function __construct(public string $name) {}
}

/**
 * @psalm-type UserOrNull = User|null
 * @return UserOrNull
 */
function findUser(bool $exists): User|null {
    return $exists ? new User("Alice") : null;
}
===expect===
