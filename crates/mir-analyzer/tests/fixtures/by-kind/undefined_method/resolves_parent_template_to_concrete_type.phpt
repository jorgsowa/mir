===description===
resolves parent template to concrete type
===file===
<?php
/** @template T */
class BaseRepo {
    /** @return T */
    public function find(): mixed { return null; }
}
class User {
    public function getName(): string { return 'Alice'; }
}
/** @extends BaseRepo<User> */
class UserRepo extends BaseRepo {}

$repo = new UserRepo();
$result = $repo->find();
$result->nonExistentMethod();
===expect===
UndefinedMethod@15:1-15:29: Method User::nonExistentMethod() does not exist
