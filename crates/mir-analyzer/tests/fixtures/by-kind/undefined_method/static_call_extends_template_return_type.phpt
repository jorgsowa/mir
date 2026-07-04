===description===
A static call's @return T resolves via the class's @extends binding, not just mixed
===file===
<?php
/** @template T */
abstract class Repository {
    /** @return T */
    public static function first() {
        throw new \RuntimeException();
    }
}

class User {}

/** @extends Repository<User> */
class UserRepository extends Repository {}

$u = UserRepository::first();
$u->onlyOnUser();
===expect===
UndefinedMethod@16:0-16:16: Method User::onlyOnUser() does not exist
