===description===
`(string) $obj` runs `__toString()`, so the cast takes its declared return type.
Covers self/named receivers, inherited and interface `__toString`, unions with
scalars, and a nullable receiver; an untyped-docblock `__toString` stays `string`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Label {
    /** @return non-empty-string */
    public function __toString(): string;
}

final class Code {
    /** @var non-empty-string */
    private string $value;

    public function __construct(self|string $value) {
        $this->value = $value instanceof self ? (string) $value : 'default';
    }

    /** @return non-empty-string */
    public function __toString(): string { return $this->value; }
}

class Base {
    /** @return numeric-string */
    public function __toString(): string { return '1'; }
}
final class Child extends Base {}

final class Plain {
    public function __toString(): string { return ''; }
}

function casts(Code $c, Child $ch, Label $l, Code|int $u, ?Code $n, Plain $p, Code|Plain $mixed): void {
    $a = (string) $c;
    /** @mir-check $a is non-empty-string */
    $b = (string) $ch;
    /** @mir-check $b is numeric-string */
    $d = (string) $l;
    /** @mir-check $d is non-empty-string */
    $e = (string) $u;
    /** @mir-check $e is non-empty-string */
    $f = (string) $n;
    /** @mir-check $f is string */
    $g = (string) $p;
    /** @mir-check $g is string */
    $h = (string) $mixed;
    /** @mir-check $h is string */
    $_ = [$a, $b, $d, $e, $f, $g, $h];
}
