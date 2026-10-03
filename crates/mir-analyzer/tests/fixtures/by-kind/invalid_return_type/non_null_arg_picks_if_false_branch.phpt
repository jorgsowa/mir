===description===
conditional return type resolves to if-false branch when non-null arg is passed
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @template T of object
 */
class Box {}

/**
 * @template T of object
 * @param class-string<T>|null $cls
 * @return ($cls is null ? Box<object> : Box<T>)
 */
function make(?string $cls = null): mixed { throw new \RuntimeException(); }

class Item {}

$b = make(Item::class);
/** @mir-check $b is Box<Item> */
echo "ok";
===expect===
