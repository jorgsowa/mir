===description===
`interface-string<T>` and a static-method receiver also bind T from a nested
list element without an InvalidArgument.
===config===
suppress=UnusedParam,MissingThrowsDocblock
===file===
<?php
interface Handler {}
class Impl implements Handler {}

/**
 * @template T
 * @param list<interface-string<T>> $ifaces
 * @return T
 */
function fromIfaces(array $ifaces) { throw new Exception(); }

class Registry {
    /**
     * @template T of object
     * @param array<class-string<T>> $classes
     * @return list<T>
     */
    public static function many(array $classes): array { return []; }
}

function test(): void {
    $h = fromIfaces([Handler::class]);
    /** @mir-check $h is Handler */
    echo get_class($h);

    $all = Registry::many([Impl::class]);
    /** @mir-check $all is list<Impl> */
    echo count($all);
}
===expect===
