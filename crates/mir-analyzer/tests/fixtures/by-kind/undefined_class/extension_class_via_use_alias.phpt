===description===
extension class via use alias
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use Swoole\Coroutine;
function f(Coroutine $x): void {}
//         ^^^^^^^^^ UndefinedClass: Class Swoole\Coroutine does not exist
