===description===
Cyclic aliases in a namespaced file degrade to mixed rather than expanding forever
or referencing a phantom class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App\Models;

/**
 * @psalm-type Node = array{value: int, children: array<int, Tree>}
 * @psalm-type Tree = array<int, Node>
 */
final class Graph {
    /** @param Tree $t */
    public function walk(array $t): void {
        /** @mir-check $t is array<int, array{value: int, children: array<int, array<int, array{value: int, children: array<int, mixed>}>>}> */
        echo 1;
    }
}
