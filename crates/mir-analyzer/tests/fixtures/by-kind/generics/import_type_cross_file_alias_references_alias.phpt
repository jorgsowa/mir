===description===
An imported alias whose body references a sibling alias in the source namespace
expands fully, also when a local alias builds on the import.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:source.php===
<?php
namespace Src;

/**
 * @psalm-type Item = array{id: int, name: string}
 * @psalm-type Items = array<int, Item>
 */
interface Source {}
===file:consumer.php===
<?php
namespace Dst;

use Src\Source;

/**
 * @psalm-import-type Items from Source
 * @psalm-type Named = array<string, Items>
 */
final class Consumer {
    /** @param Items $items */
    public function __construct(public array $items) {}

    /** @param Named $named */
    public function named(array $named): void {
        /** @mir-check $named is array<string, array<int, array{id: int, name: string}>> */
        echo 1;
    }
}

new Consumer([1 => ['id' => 1, 'name' => 'a']]);
new Consumer('nope');
//           ^^^^^^ InvalidArgument: Argument $items of Dst\Consumer::__construct() expects 'array<int, array{'id': int, 'name': string}>', got '"nope"'
===expect===
