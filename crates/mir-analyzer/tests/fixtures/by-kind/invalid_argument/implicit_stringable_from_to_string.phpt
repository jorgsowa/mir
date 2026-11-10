===description===
A class or interface declaring `__toString()` implicitly implements
`Stringable`, and so does an interface extending one.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

class Name {
    public function __toString(): string { return 'n'; }
}

interface Label {
    public function __toString(): string;
}

interface ChildLabel extends Label {}

interface Plain {
    public function name(): string;
}

function takesStringable(\Stringable $value): void {}

function run(Name $n, Label $l, ChildLabel $c, Plain $p): void {
    takesStringable($n);
    takesStringable($l);
    takesStringable($c);
    takesStringable($p);
//                  ^^ InvalidArgument: Argument $value of takesStringable() expects 'Stringable', got 'Plain'
}
