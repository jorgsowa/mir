===description===
FALSE POSITIVE reproducer. Calling a method annotated @inheritdoc should resolve
to the parent's declared return type, not mixed. The @mir-check below would fail
if the return type remained mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
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
