===description===
callable(Enum::Case) accepts a callback whose parameter is the enum or one of its interfaces
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

interface HasLabel {}
enum Err: string implements HasLabel {
    case A = 'a';
    case B = 'b';
}

/** @param callable(Err::A): mixed $f */
function onA(callable $f): void {}

/** @param \Closure(Err::A|Err::B): mixed $f */
function onEither(\Closure $f): void {}

final class Bus {
    /** @param callable(Err::A): mixed $f */
    public function on(callable $f): void {}
}

onA(fn(Err $e) => null);
onA(fn(HasLabel $e) => null);
onA(function (Err $e): void {});
onEither(fn(Err $e) => null);
(new Bus())->on(fn(Err $e) => null);
