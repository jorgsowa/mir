===description===
array_merge of arrays with string keys keeps the keys and unions the values
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array<string, int> $map
 * @param array<string, string> $other
 * @param array $untyped
 * @param non-empty-array<string, int> $full
 */
function merge(array $map, array $other, array $untyped, array $full): void {
    $literals = array_merge(['a' => 1], ['b' => 2]);
    /** @mir-check $literals is non-empty-array<"a"|"b", 1|2> */

    $typed = array_merge($map, $other);
    /** @mir-check $typed is array<string, int|string> */

    $withEmpty = array_merge($map, []);
    /** @mir-check $withEmpty is array<string, int> */

    $nonEmpty = array_merge($map, $full);
    /** @mir-check $nonEmpty is non-empty-array<string, int> */

    $unknown = array_merge($map, $untyped);
    /** @mir-check $unknown is array */

    $recursive = array_merge_recursive($map, $other);
    /** @mir-check $recursive is array */
}
