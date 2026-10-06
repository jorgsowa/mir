===description===
Template param with no bound does not fire InvalidThrow — unbounded T defaults to mixed, which cannot be statically rejected
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @param T $e
 */
function rethrow($e): never {
    throw $e;
}
