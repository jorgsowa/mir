===description===
A cross-file imported alias is expanded in return types, property types and
`@var` annotations, and honours the `as` rename.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:repo.php===
<?php
namespace Core\Repository;

/** @psalm-type Row = array{id: int, name: string} */
class Rows {}
===file:svc.php===
<?php
namespace Core\Service;

use Core\Repository\Rows;

/** @psalm-import-type Row as CatalogRow from Rows */
class Svc {
    /** @var CatalogRow */
    public array $row = ['id' => 1, 'name' => 'x'];

    /** @return CatalogRow */
    public function make(): array { return ['id' => 1, 'name' => 'x']; }

    public function run(): void {
        $made = $this->make();
        /** @mir-check $made is array{'id': int, 'name': string} */
        echo 1;
        $prop = $this->row;
        /** @mir-check $prop is array{'id': int, 'name': string} */
        echo 1;
        /** @var CatalogRow $local */
        $local = $GLOBALS['x'];
        /** @mir-check $local is array{'id': int, 'name': string} */
        echo 1;
    }
}
