===description===
callable(Enum::Case) still rejects callbacks whose parameter can't take the case
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

enum Err { case A; case B; }
enum Other { case X; }

/** @param callable(Err::A): mixed $f */
function onA(callable $f): void {}

onA(fn(int $e) => null);
//  ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of onA() expects 'callable whose parameter #1 accepts App\Err::A', got 'callable whose parameter #1 only accepts int'
onA(fn(Other $e) => null);
//  ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of onA() expects 'callable whose parameter #1 accepts App\Err::A', got 'callable whose parameter #1 only accepts App\Other'
===expect===
