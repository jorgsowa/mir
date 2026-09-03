===description===
FALSE POSITIVE reproducer. Calling a method annotated @inheritdoc should resolve
to the parent's declared return type, not mixed. The @mir-check below would fail
if the return type remained mixed.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php
class User {}

abstract class BaseRepository {
    /** @return User */
    abstract public function find(int $id): mixed;
}

class ConcreteRepository extends BaseRepository {
    /** @inheritdoc */
    public function find(int $id): mixed {
        return new User();
    }
}

function test(ConcreteRepository $repo): void {
    $user = $repo->find(1);
    /** @mir-check $user is User */
    echo get_class($user);
}
===expect===
