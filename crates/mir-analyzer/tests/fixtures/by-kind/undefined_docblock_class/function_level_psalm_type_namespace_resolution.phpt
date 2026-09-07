===description===
@psalm-type alias with a class type in a namespace resolves the class correctly
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class User {
    public function getName(): string { return ''; }
}

/**
 * @psalm-type MaybeUser = User|null
 * @param MaybeUser $u
 * @return string
 */
function getNameOrEmpty(User|null $u): string {
    return $u !== null ? $u->getName() : '';
}
===expect===
