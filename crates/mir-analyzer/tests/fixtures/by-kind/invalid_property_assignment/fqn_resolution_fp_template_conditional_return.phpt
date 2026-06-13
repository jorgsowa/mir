===description===
FP guard: (T is null ? list<array> : array<array>) — template conditional without $ is parsed correctly and neither T nor array is namespace-qualified (regression: both were being qualified to Namespace\T and Namespace\array)
===config===
suppress=UnusedVariable
===file:Lib/Db.php===
<?php
namespace Lib;

class Db {
    /**
     * @template TKey of non-empty-string|null
     * @param TKey $key
     * @return (TKey is null ? list<array> : array<array>)
     */
    public function fetch(mixed $key): array {
        return [];
    }
}
===file:App/UseIt.php===
<?php
namespace App;

use Lib\Db;

$db = new Db();
// The return type must be stored as TConditional — verify it contains no namespace-qualified
// garbage like Lib\TKey or Lib\array by checking the type display is a proper conditional form.
$result = $db->fetch(null);
/** @mir-check $result is ($TKey is null ? list<array<mixed, mixed>> : array<int, array<mixed, mixed>>) */
===expect===
Db.php: UnusedParam@10:27-10:37: Parameter $key is never used
