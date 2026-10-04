===description===
A @psalm-type body referencing another alias expands fully in a namespaced file,
through chains and nested generic/shape positions, and still rejects bad values.
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
 * @psalm-type Item = array{id: int, name: string}
 * @psalm-type Items = array<int, Item>
 * @psalm-type Index = array<string, Items>
 * @psalm-type Wrapper = array{items: Items, first: Item}
 */
final class Box {
    /** @param Items $items */
    public function items(array $items): void {
        /** @mir-check $items is array<int, array{id: int, name: string}> */
        echo 1;
    }

    /** @param Index $index */
    public function index(array $index): void {
        /** @mir-check $index is array<string, array<int, array{id: int, name: string}>> */
        echo 1;
    }

    /** @param Wrapper $w */
    public function wrapper(array $w): void {
        /** @mir-check $w is array{items: array<int, array{id: int, name: string}>, first: array{id: int, name: string}} */
        echo 1;
    }

    /** @param Items $items */
    public function __construct(public array $items) {}
}

new Box([1 => ['id' => 1, 'name' => 'a']]);
new Box('nope');
//      ^^^^^^ InvalidArgument: Argument $items of App\Models\Box::__construct() expects 'array<int, array{'id': int, 'name': string}>', got '"nope"'
===expect===
