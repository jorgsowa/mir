===description===
An inline `@var` resolves `Enum::CASE` in a generic argument against the file's imports
===file===
<?php
namespace N {
    enum Id: string { case INT = 'int'; }

    /** @template U of Id */
    class Box {
        /** @return U */
        public function id(): Id { return Id::INT; }
    }
}

namespace App {
    use N\Box;
    use N\Id;

    function f(mixed $k): void {
        /** @var Box<Id::INT> $box */
        $box = $k;
        $id = $box->id();
        /** @mir-check $id is N\Id::INT */
        echo $id->value;
    }
}
