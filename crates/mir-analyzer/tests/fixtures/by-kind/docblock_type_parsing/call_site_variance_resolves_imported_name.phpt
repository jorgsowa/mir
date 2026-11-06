===description===
The type after `covariant` resolves through the file's imports.
===file===
<?php
namespace Models {
    class Model { public function save(): void {} }
    class User extends Model {}
    /** @template TModel of Model */
    class Coll {
        /** @return TModel|null */
        public function first() { return null; }
    }
}
namespace App {
    use Models\Coll;
    use Models\Model;
    /** @param Coll<covariant Model> $r */
    function f($r): void {
        $r->first()?->save();
    }
    /** @param Coll<\Models\User> $u */
    function g($u): void {
        f($u);
    }
}
