===description===
Reading a key a shape does not list gives the remainder type of `...<Foo>`; a dynamic key gives the listed value types and the remainder.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {}

/** @param array{name: string, ...<Foo>} $row */
function readsRow(array $row, string $key): void {
    if (isset($row['extra'])) {
        /** @mir-check $row['extra'] is Foo */
        $_ = 1;
    }
    /** @mir-check $row[$key] is Foo|string */
    $_ = 1;
    /** @mir-check $row is array{'name': string, ...<Foo>} */
    $_ = 1;
}
