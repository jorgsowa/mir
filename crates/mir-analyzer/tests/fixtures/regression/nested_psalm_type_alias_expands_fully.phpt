===description===
A @psalm-type alias whose body references another same-file alias must
expand through to that alias's own definition instead of stopping one
level short (`type UserId = Id; type Id = int;` should make `UserId`
resolve all the way to `int`, not stop at the unexpanded `Id`).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @psalm-type Id = int
 * @psalm-type UserId = Id
 */
class Repo {
    /**
     * @param UserId $id
     */
    public function find($id) {
        /** @mir-check $id is int */
        echo "ok";
    }
}
===expect===
