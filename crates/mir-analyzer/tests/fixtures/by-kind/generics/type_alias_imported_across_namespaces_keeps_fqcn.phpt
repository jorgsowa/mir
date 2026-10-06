===description===
A @psalm-import-type alias whose body names a class resolved in the defining
file's namespace keeps that class when used from a different namespace.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:contract.php===
<?php
namespace Lib\Contract;

interface Item {}
===file:repo.php===
<?php
namespace Lib\Repo;

use Lib\Contract\Item;

/** @psalm-type Row = array{item: Item, tags: list<Item>} */
class Rows {}
===file:svc.php===
<?php
namespace App\Service;

/** @psalm-import-type Row from \Lib\Repo\Rows */
class Svc {
    /** @param Row $r */
    public function run(array $r): void {
        /** @mir-check $r is array{'item': Lib\Contract\Item, 'tags': list<Lib\Contract\Item>} */
        echo 1;
    }
}
