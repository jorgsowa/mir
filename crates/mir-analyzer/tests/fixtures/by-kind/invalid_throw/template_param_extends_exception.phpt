===description===
Throwing a @template T of Exception does not fire InvalidThrow
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of Exception
 * @param T $e
 */
function rethrow($e): never {
    throw $e;
}
===expect===
