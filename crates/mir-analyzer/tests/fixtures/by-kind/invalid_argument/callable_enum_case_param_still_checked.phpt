===description===
callable(Enum::Case) still rejects callbacks whose parameter can't take the case
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
namespace App;

enum Err { case A; case B; }
enum Other { case X; }

/** @param callable(Err::A): mixed $f */
function onA(callable $f): void {}

onA(fn(int $e) => null);
onA(fn(Other $e) => null);
===expect===
InvalidArgument@10:4-10:22: Argument $f of onA() expects 'callable whose parameter #1 accepts App\Err::A', got 'callable whose parameter #1 only accepts int'
InvalidArgument@11:4-11:24: Argument $f of onA() expects 'callable whose parameter #1 accepts App\Err::A', got 'callable whose parameter #1 only accepts App\Other'
