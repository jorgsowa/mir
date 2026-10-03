===description===
FALSE POSITIVE reproducer. @inheritdoc should inherit the parent's @return type so
a correct implementation does not emit InvalidReturnType.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class User {}

interface UserRepository {
    /** @return User */
    public function find(int $id): mixed;
}

class DatabaseUserRepository implements UserRepository {
    /** @inheritdoc */
    public function find(int $id): mixed {
        return new User();
    }
}
===expect===
