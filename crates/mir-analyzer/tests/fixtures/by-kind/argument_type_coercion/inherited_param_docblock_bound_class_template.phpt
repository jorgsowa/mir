===description===
A parent's `@param T` refines an override's native param through the class's
`@implements Runner<Concrete>` binding.
===file===
<?php
abstract class Handle {}
final class WinHandle extends Handle { public int $code = 0; }

/** @template T of Handle */
interface Runner {
    /** @param T $h */
    public function join(Handle $h): int;
}

/** @implements Runner<WinHandle> */
final class WinRunner implements Runner {
    public function join(Handle $h): int {
        return $h->code;
    }
}

/** @template T of Handle */
interface Unbound {
    /** @param T $h */
    public function join(Handle $h): int;
}

final class RawRunner implements Unbound {
    public function join(Handle $h): int {
        /** @mir-check $h is Handle */
        return 1;
    }
}
