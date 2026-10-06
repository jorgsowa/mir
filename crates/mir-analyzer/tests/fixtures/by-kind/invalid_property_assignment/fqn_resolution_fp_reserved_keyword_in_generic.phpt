===description===
FP guard: PHP reserved type keywords (array, list) inside generic type params must not be namespace-qualified
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Lib/Mapper.php===
<?php
namespace Lib;

/**
 * @template TIn
 * @template TOut
 */
class Mapper {
    /**
     * @template T
     * @param list<T> $items
     * @return list<array>
     */
    public function toArrays(array $items): array {
//                           ^^^^^^^^^^^^ UnusedParam: Parameter $items is never used
        return [];
    }
}
===file:App/UseIt.php===
<?php
namespace App;

use Lib\Mapper;

$mapper = new Mapper();
$result = $mapper->toArrays([1, 2, 3]);
/** @mir-check $result is list<array> */
===expect===
