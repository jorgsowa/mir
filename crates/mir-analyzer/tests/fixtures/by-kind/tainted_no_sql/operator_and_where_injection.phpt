===description===
A request value that may be an array turns a MongoDB filter into an operator
query, and a tainted `$where` clause runs as JavaScript.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MongoDB {
    class Collection {
        public function findOne(array|object $filter = [], array $options = []): ?object { return null; }
        public function deleteMany(array|object $filter, array $options = []): void {}
    }
}

namespace MongoDB\Driver {
    final class Query {
        public function __construct(array|object $filter, ?array $queryOptions = null) {}
    }
}

namespace App {
    use MongoDB\Collection;
    use MongoDB\Driver\Query;

    function login(Collection $users): void {
        $users->findOne(['user' => $_POST['user'], 'pass' => $_POST['pass']]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedNoSql: Tainted NoSQL query — possible NoSQL injection
    }

    function remove(Collection $items): void {
        $owner = $_GET['owner'];
        $items->deleteMany(['owner' => $owner]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedNoSql: Tainted NoSQL query — possible NoSQL injection
    }

    function search(): void {
        $query = new Query(['$where' => "this.name == '" . $_GET['name'] . "'"]);
//                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedNoSql: Tainted NoSQL query — possible NoSQL injection
    }
}
