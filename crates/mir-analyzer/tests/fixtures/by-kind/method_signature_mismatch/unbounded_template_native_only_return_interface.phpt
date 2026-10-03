===description===
A native-only implementation of an interface method whose `@return T` is bound
by `@implements` is not compared against the bound type: the child never
restated the docblock, and PHP accepts its native hint.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Wither {
    /** @return T */
    public function with(array $a): Wither;
}

/** @implements Wither<Shade> */
final class Shade implements Wither {
    public function with(array $a): Wither {
        return new self();
    }
}

function use_it(Shade $s): void {
    $r = $s->with([]);
    /** @mir-check $r is Wither */
    echo get_class($r);
}
===expect===
