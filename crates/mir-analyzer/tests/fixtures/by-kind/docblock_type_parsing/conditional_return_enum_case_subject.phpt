===description===
A conditional return on enum-case subjects picks the branch for a known case, and the union of branches for a wider enum; an omitted argument uses its literal default.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib;

enum Key: string {
    case A = 'a';
    case B = 'b';
    case C = 'c';
}

final class Store {
    /**
     * @template T of Key
     * @param T $k
     * @return (
     *   T is Key::A ? int :
     *   T is Key::B ? string :
     *   null
     * )|null
     */
    public function get(Key $k): mixed {
        return null;
    }

    /**
     * @return ($flag is true ? int : string)
     */
    public function flag(bool $flag = true): int|string {
        return 1;
    }
}

function check(Store $s, Key $any): void {
    /** @mir-check $s->get(Key::A) is int|null */
    $s->get(Key::A);
    /** @mir-check $s->get(Key::B) is string|null */
    $s->get(Key::B);
    /** @mir-check $s->get(Key::C) is null */
    $s->get(Key::C);
    /** @mir-check $s->get($any) is int|string|null */
    $s->get($any);
    /** @mir-check $s->flag() is int */
    $s->flag();
    /** @mir-check $s->flag(false) is string */
    $s->flag(false);
}
===expect===
