===description===
`@psalm-import-type` from a class in another file/namespace expands to the
source alias body instead of a same-named class in the importing namespace.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:repo.php===
<?php
namespace Core\Repository;

/** @psalm-type Row = array{id: int, name: string} */
class Rows {
    /** @return Row */
    public function first(): array { return ['id' => 1, 'name' => 'a']; }
}
===file:svc.php===
<?php
namespace Core\Service;

use Core\Repository\Rows;

/** @psalm-import-type Row from Rows */
class Svc {
    /** @param Row $r */
    public function take(array $r): void {}

    /** @param list<Row> $rs */
    public function each(array $rs): void {
        foreach ($rs as $r) {
            /** @mir-check $r is array{'id': int, 'name': string} */
            echo 1;
        }
    }

    public function run(Rows $rows): void {
        $this->take($rows->first());
        $this->take(['id' => 1, 'name' => 'x']);
        $this->take(['id' => 'x']);
//                  ^^^^^^^^^^^^^ InvalidArgument: Argument $r of take() expects 'array{'id': int, 'name': string}', got 'array{'id': "x"}'
    }
}
