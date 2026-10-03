===description===
`$x instanceof self`/`static`/`parent` must narrow $x to the actual current
class, not to a bogus type literally named "self"/"static"/"parent" — those
are keywords `db::resolve_name` deliberately leaves unresolved (it has no
class context), so narrowing must resolve them itself before matching.
Previously the narrowed type became e.g. `TNamedObject{fqcn:"self"}`, a
nonexistent class; member/argument checks against an unresolvable class are
treated permissively, so real bugs on the narrowed value went undetected.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Marker {}

class Base implements Marker {
    public function needsInt(int $n): void {}

    public function checkSelf(Marker $x): void {
        if ($x instanceof self) {
            $x->needsInt("not an int");
//                       ^^^^^^^^^^^^ InvalidArgument: Argument $n of needsInt() expects 'int', got '"not an int"'
        }
    }

    public function checkStatic(Marker $x): void {
        if ($x instanceof static) {
            $x->needsInt("not an int");
//                       ^^^^^^^^^^^^ InvalidArgument: Argument $n of needsInt() expects 'int', got '"not an int"'
        }
    }
}

class Derived extends Base {
    public function checkParent(Marker $x): void {
        if ($x instanceof parent) {
            $x->needsInt("not an int");
//                       ^^^^^^^^^^^^ InvalidArgument: Argument $n of needsInt() expects 'int', got '"not an int"'
        }
    }
}
===expect===
