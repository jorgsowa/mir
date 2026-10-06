===description===
An inline @var nested inside non-empty-array, a shape, or an intersection resolves imported class names.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    class Item {}
    class Other {}
}

namespace App {
    use Lib\Item;
    use Lib\Other;

    /** @return non-empty-array<string, non-empty-array<int, Item>> */
    function nonEmptyNested(): array {
        $g = [];
        /** @var non-empty-array<string, non-empty-array<int, Item>> $g */
        return $g;
    }

    function resolvedType(): void {
        /** @var non-empty-array<string, non-empty-array<int, Item>> $g */
        $g = [];
        /** @mir-check $g is non-empty-array<string, non-empty-array<int, Lib\Item>> */
        echo count($g);
    }

    /** @return array{a: non-empty-array<int, Item>} */
    function shapeValue(): array {
        $g = [];
        /** @var array{a: non-empty-array<int, Item>} $g */
        return $g;
    }

    /** @return non-empty-array<int, Item&\Countable> */
    function intersectionValue(): array {
        $g = [];
        /** @var non-empty-array<int, Item&\Countable> $g */
        return $g;
    }

    /** @return non-empty-array<string, non-empty-array<int, Item>> */
    function wrongClass(): array {
        $g = [];
        /** @var non-empty-array<string, non-empty-array<int, Other>> $g */
        return $g;
//      ^^^^^^^^^^ InvalidReturnType: Return type 'non-empty-array<string, non-empty-array<int, Lib\Other>>' is not compatible with declared 'non-empty-array<string, non-empty-array<int, Lib\Item>>'
    }
}
