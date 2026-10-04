===description===
Alias-to-alias expansion in the global namespace (no qualification involved).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-type Item = array{id: int}
 * @psalm-type Items = array<int, Item>
 */
final class GlobalBox {
    /** @param Items $items */
    public function items(array $items): void {
        /** @mir-check $items is array<int, array{id: int}> */
        echo 1;
    }
}
===expect===
