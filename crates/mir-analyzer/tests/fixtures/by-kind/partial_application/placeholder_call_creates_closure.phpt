===description===
A call with `?` placeholders creates a Closure for functions, methods and static methods.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function add(int $a, int $b): int { return $a + $b; }
class K {
    public function m(int $a): string { return (string) $a; }
    public static function s(int $a): int { return $a; }
}

function takesClosure(Closure $c): void { $c(1); }
function takesString(string $s): void { echo $s; }

function ok(K $k): void {
    takesClosure(add(1, ?));
    takesClosure(add(?, ?));
    takesClosure($k->m(?));
    takesClosure(K::s(?));
    $f = add(?, 2);
    echo $f(1);
}

function bad(K $k): void {
    takesString(add(1, ?));
    takesString($k->m(?));
    takesString(K::s(?));
    takesInt(add(1, 2));
}
function takesInt(int $i): void { echo $i; }
===expect===
InvalidArgument@21:16-21:25: Argument $s of takesString() expects 'string', got 'Closure(int): int'
InvalidArgument@22:16-22:24: Argument $s of takesString() expects 'string', got 'Closure(int): string'
InvalidArgument@23:16-23:23: Argument $s of takesString() expects 'string', got 'Closure(int): int'
