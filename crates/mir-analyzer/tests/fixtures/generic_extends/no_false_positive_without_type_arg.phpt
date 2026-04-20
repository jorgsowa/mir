===source===
<?php
/** @template T */
class BaseRepo {
    /** @return T */
    public function find(): mixed { return null; }
}
class UserRepo extends BaseRepo {}

$repo = new UserRepo();
$result = $repo->find();
===expect===
