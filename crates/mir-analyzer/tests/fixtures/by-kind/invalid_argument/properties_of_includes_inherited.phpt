===description===
`properties-of` covers inherited properties and resolves an imported class name; an unknown class stays a string-keyed array.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Domain {
    class Entity {
        public int $id = 0;
    }

    final class Product extends Entity {
        public string $sku = '';
    }
}

namespace App {
    use Domain\Product;

    /** @param properties-of<Product> $fields */
    function save(array $fields): void {}

    /** @param properties-of<Vendor\Unknown> $fields */
    function saveUnknown(array $fields): void {}

    save(['sku' => 'A-1', 'id' => 7]);
    save(['sku' => 'A-1', 'id' => 'seven']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $fields of save() expects 'array{'sku': string, 'id': int}', got 'array{'sku': "A-1", 'id': "seven"}'
    save(['sku' => 'A-1']);
//       ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $fields of save() expects 'array{'sku': string, 'id': int}', got 'array{'sku': "A-1"}'
    saveUnknown(['anything' => 1]);
    saveUnknown(false);
//              ^^^^^ InvalidArgument: Argument $fields of saveUnknown() expects 'array<string, mixed>', got 'false'
}
