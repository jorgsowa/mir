===description===
MixedArrayOffset fires when a mixed-typed function parameter is used as the array key
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param mixed $key
 */
function lookup($key): void {
    $arr = ['a' => 1, 'b' => 2, 'c' => 3];
    echo $arr[$key];
//            ^^^^ MixedArrayOffset: Mixed type used as array offset
}
