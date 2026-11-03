===description===
`get_object_vars`, `(array)`, `foreach` over the object and `serialize` read every property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Vars {
    private int $a = 1;
    public function all(): array { return get_object_vars($this); }
}

final class Cast {
    private int $b = 1;
    public function all(): array { return (array) $this; }
}

final class Loop {
    private int $c = 1;
    public function all(): array {
        $out = [];
        foreach ($this as $k => $v) {
//                              ^^ MixedAssignment: Variable $v is assigned a mixed type
            $out[$k] = $v;
        }
        return $out;
    }
}

final class Ser {
    private int $d = 1;
    public function dump(): string { return serialize($this); }
}

final class Unrelated {
    private int $e = 1;
//  ^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Unrelated::$e is never read
    public function vars(Vars $v): array { return get_object_vars($v); }
}

(new Vars())->all();
(new Cast())->all();
(new Loop())->all();
(new Ser())->dump();
(new Unrelated())->vars(new Vars());
