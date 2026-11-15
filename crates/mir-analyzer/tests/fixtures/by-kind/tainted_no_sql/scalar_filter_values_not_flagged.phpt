===description===
Filter values cast to a scalar can't carry an operator, and filters built from
program data are fine.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MongoDB {
    class Collection {
        public function findOne(array|object $filter = [], array $options = []): ?object { return null; }
    }
}

namespace App {
    use MongoDB\Collection;

    function login(Collection $users, string $tenant): void {
        $users->findOne(['user' => (string) $_POST['user'], 'tenant' => $tenant]);
        $name = (string) $_GET['name'];
        $users->findOne(['name' => $name, 'age' => (int) $_GET['age']]);
        $users->findOne(['$where' => 'this.active == true']);
    }
}
