===description===
A generic (parameterized) @psalm-type alias name kept the `<T>` suffix
verbatim in the stored name, so even a BARE (non-parameterized) use site
never matched by the plain name — the alias was silently 100% dead, not
just unable to substitute its template param.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type ListOf<T> = array<int, T>
 */
class Repo {
    /**
     * @param ListOf $x
     */
    public function find($x) {
        /** @mir-check $x is array<int, T> */
        echo "ok";
    }
}
===expect===
